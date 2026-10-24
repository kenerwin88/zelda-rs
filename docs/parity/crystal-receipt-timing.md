# Crystal receipt timing

Measured from `6dc983f29df97103eab584e67047ea802e5d826d` on `codex/crystal-timing`.

**The original seven live-SPC probes are shorter on screen, not merely earlier in logical iterations. The unexpected sign comes from an invalid foreign-seed audio bank. That seed bug is repaired; the requested complete native phase-duration match remains unresolved.** No duration table, ledger playback, new cycle model, or artificial hold was added.

## Measurement and limits

The eight probes start before song `$13`, use the same ledger inputs and cartridge RNG, and never reseed inside their windows. `--windows` is a reusable harness option for disjoint uninterrupted probes in one read-only image-stream pass. Both native configurations use the same engine. This is a CPU-only phase/state measurement; no GPU, Snes9x, cached-A/V execution, or pixel acceptance is claimed.

An iteration number in the tables labels the state **after** that iteration. The original display boundary is its immutable `picture_host`; the native boundary advances one frame per iteration. Elapsed times start at each side’s first observed `$13` command boundary. The command is sampled by the engine’s closing NMI but appears among the following ledger iteration’s opening-NMI writes, explaining the one-index difference in the issue rows. Message open/close means entry/exit of message routing `$0E/02`, not a GPU measurement of the first border pixel. A poll row is the iteration containing the observed value transition; the ledger does not retain that read’s subframe timestamp.

`N` is the requested counted interval, `Δiterations + Σlag_nmis`, over the post-iteration boundaries. `H` is the actual elapsed presentation-host interval. **N is not always H:** a bank-upload interval can advance hosts while NMI is disabled, so no accepted lag NMI is counted. For example, original iteration **383631** advances the next opening host from **416394 to 416478** (84 frames), with **59 lag NMIs**: `1 + 59` misses **24** frames. It contains 26,929 APU writes. `crates/snes/examples/logical_frame_ledger.rs:623` records accepted openings and `:636` counts accepted lag NMIs; `crates/zelda3/src/spc_driver_clock.rs:1036` describes the upload caller restoring `$4200`. The difference is not permission to treat all missing time as lag.

Original SPC instruction timestamps are absent from this ledger. Its elapsed **DSP output sample clock** is recoverable from the immutable cache’s actual per-host audio sample counts: `original-audio-boundaries.json` records those sums at every original boundary. Native `bank.jsonl` retains the actual SPC summary, including absolute APU cycles. In the detailed tables, original `S` is that elapsed cached DSP-output count; repaired native `S` is `(APU_cycle_boundary − APU_cycle_at_song_issue) / 32`. Baseline native `S` follows from the observed fixed callback count. Both native columns have exactly **534 × elapsed frames** samples. These are boundary sample-clock coordinates, not invented original SPC instruction timestamps or exact intra-frame read times.

## Total elapsed time and plain answer

Times include the receipt, maiden messages and exit/iris to the first stable outdoor state. `Original H` uses presentation hosts, not the incomplete NMI sum. A negative difference means the native scene is shorter. Seconds are NTSC frame equivalents at 60.0988 Hz, not a wall-clock/GPU playback benchmark. Human button presses remain indexed to logical iterations, so the total includes time spent waiting for the recorded dismissal presses.

| Scene / seed | Base stable i | Original stable i | Base frames | Original N | Original H | Base − original H | Repaired stable i / frames | Repaired − original H |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |
| Pendant control (Eastern Palace) / 94000 | 95163 | 95163 | 948 | 1061 | 1084 | **shorter 136 f (2.263 s)** | 95163 / 948 | -136 f (-2.263 s) |
| Crystal 1 — Palace of Darkness / 380000 | 383469 | 383667 | 3227 | 3702 | 3726 | **shorter 499 f (8.303 s)** | 383667 / 3425 | -301 f (-5.008 s) |
| Crystal 2 — Swamp Palace / 460000 | 465767 | 465847 | 3779 | 4160 | 4184 | **shorter 405 f (6.739 s)** | 465848 / 3860 | -324 f (-5.391 s) |
| Crystal 3 — Skull Woods / 626000 | 629663 | 629876 | 3046 | 3514 | 3539 | **shorter 493 f (8.203 s)** | 629876 / 3259 | -280 f (-4.659 s) |
| Crystal 4 — Thieves’ Town / 734000 | 738023 | 738219 | 2795 | 3209 | 3233 | **shorter 438 f (7.288 s)** | 738219 / 2991 | -242 f (-4.027 s) |
| Crystal 5 — Ice Palace / 880000 | 884125 | 884358 | 3031 | 3519 | 3543 | **shorter 512 f (8.519 s)** | 884358 / 3264 | -279 f (-4.642 s) |
| Crystal 6 — Misery Mire / 1064000 | 1066932 | 1066994 | 2837 | 3123 | 3147 | **shorter 310 f (5.158 s)** | 1066994 / 2899 | -248 f (-4.127 s) |
| Crystal 7 — Turtle Rock / 1316000 | 1320740 | 1320931 | 3044 | 3475 | 3500 | **shorter 456 f (7.588 s)** | 1321195 / 3499 | -1 f (-0.017 s) |

The zero logical shift of the pendant did **not** establish equal on-screen time. For Turtle Rock after the repair, the near-equal total is accidental: its last message waits for a later input edge, offsetting shorter earlier phases. Its phase durations still fail the requested one-or-two-frame criterion.

## Cause and native repair

1. **Wrong bank in the probes.** `set_assets` initializes the native SPC with asset 0 (`crates/zelda3/src/zelda_rtl.rs:14544`; `crates/zelda3/src/spc_driver_clock.rs:516`). Every receipt seed carries resident-bank selector `$0136=1`, but the old `seed_memory_from_foreign_image` restored WRAM and semantic music ports without loading dungeon asset 1 into SPC RAM. Its apparent live receipt song therefore was not the dungeon receipt song. The crystal’s first song poll returned zero, at only 1–3 native frames after issue, versus 638–639 original frames. The pendant likewise opened its message hundreds of frames early but waited for the same later dismissal input. This is why the naive clock argument had the wrong sign: it assumed the same resident song data.

   The native repair at `crates/zelda3/src/zelda_rtl.rs:14921` loads the resident bank from the seed’s existing selector through `load_song_bank`, before restoring the song. This installs native song assets, never ledger durations, poll values, SPC snapshots or phase tables. It repairs foreign seeding, not continuous play’s cutscene pacing. With the repair, the receipt song lasts 637–638 native frames, within 0–1 elapsed frames of the 638–639-frame reference; observed zero-poll iteration indices remain 1–2 earlier.

2. **SPC rate is a small effect.** `audio.rs:983` normalizes callbacks to 534 samples and `spc_driver_clock.rs:592` advances 32 SPC clocks per sample. At 60.0988 Hz this is 32,092.7592 samples/s; `32000 / 60.0988 = 532.4566` samples/frame. The pinned software clock also uses its explicit CPU/APU ratio (`spc_driver_clock.rs:40`), so nominal 32 kHz alone is not an exact substitute for that reference. This small rate difference can move this song’s end by roughly a frame or two. It cannot explain an immediate zero poll, nor hundreds of missing scene frames. No global audio-rate change was made.

   For crystal 1 the original zero-poll boundary is **340,132 DSP output samples** after issue; the repaired native boundary is **340,158 SPC/DSP samples** after issue. The song-clock interval is almost identical in samples while occupying 638 original versus 637 native frames. That is the small clock-rate effect, not the earlier 638-iteration skip. **Every original crystal’s song-issue-to-zero interval has zero lag NMIs:** logical iterations, N and H are all 638 or 639. The naive lag-clock argument therefore does not apply to this particular phase in the first place. Native poll `$01` is the engine’s cached is-playing boolean derived from the SPC acknowledgement; the cartridge exposes song IDs `$15/$13`. The callers compare zero/nonzero only.

3. **Polygon growth is not faster.** From native thread activation to the first maiden message the measured intervals match exactly: 469, 469, 472, 472, 469, 472, 471 frames for crystals 1–7. This holds before and after the seed repair. Maiden AI 2/3/5 boundaries below agree on the same growth/emergence/palette intervals. During dialogue the existing scheduler credits extra worker phases for cartridge lag (`zelda_rtl/rtl_poly.rs:374`), but does not spend corresponding audio/display frames. That explains worker workspace differences without making crystal growth itself the missing 200-frame phase.

4. **Dialogue and exit work are genuinely compressed.** Lagless VWF removes its per-frame work limit (`messaging.rs:7889`, `cycles_left = u32::MAX`). ROM-less scroll drains a whole `scroll_speed + 1` group in one call (`messaging.rs:9297`), while the source’s retained scroll can spread a five-pixel group across `2+2+1` passes (`messaging.rs:9246`). Original message intervals contain 7–8-lag-NMI character-map setup calls and repeated two-lag-NMI glyph/scroll calls. This is rendering/scroll work, not proof that every ordinary typewriter delay is wrong. Exit loading dispatches the native properties/overlay/build work through `overworld.rs:167`; original scene exit-to-stable is 167–176 host frames versus **37 native frames**. Its 16-step iris also has original lag NMIs; the native table calculation does not spin on the raster. The explicit event tables separate those phases.

5. **The net logical shift is input-dependent.** Before the seed repair, starting dialogue about 638 frames early makes the message interval longer while it waits for recorded input. This absorbs much of the song error, leaving shifts of only 62–233 iterations. Restoring the bank removes nearly all those shifts without restoring the original elapsed dialogue or loading time. In the repaired Turtle Rock probe, the two-index entry change misses a dismissal window: the last message closes at 1320926 instead of 1320664. This adds 264 logical iterations at the stable boundary and happens to mask the earlier frame savings.

**Unresolved native pacing:** this patch does not meet the complete on-screen phase-duration target. Matching the remaining source rendering/scroll/load timing requires native retained work/publication phases driven by an independently justified work cadence, including the bank-upload await, text-copy progress, and loading/iris publication. A blanket delay, per-crystal hold, replay of the lag counts, or a new cycle budget would hide the measured causes and violate the requested constraints. None was installed. The preserved matching save outcomes below are not being presented as timing equivalence.

## Phase durations before and after the seed repair

Original cells are `iterations / counted N / actual H`; base/repaired cells are real native frames. Message interval runs from the first speech opening through the victory/warp entry `$16/00`, so it includes intervening player-controlled dismissal waits. `$16/00..04` are heal, start/run/close victory spin, and fade (`misc.rs:1273`); those warp phases match their original real-frame durations. `$08/00..02` then performs the loading work, followed by `$10` iris opening.

| Crystal / seed | Song → zero: original ; base ; repaired | Thread → speech: original / repaired | Messages → victory: original ; base ; repaired | Victory → load: original / repaired | Load → stable: original / repaired |
| --- | --- | --- | --- | --- | --- |
| 380000 | 638 / 638 / 638 ; base **1** ; repaired **637** | 469 / 469 / 469 ; **469** | 2052 / 2218 / 2218 ; base **2492** ; repaired **2054** | 153 / 153 / 153 ; **153** | 37 / 143 / 167 ; **37** |
| 460000 | 639 / 639 / 639 ; base **1** ; repaired **638** | 469 / 469 / 469 ; **469** | 2484 / 2674 / 2674 ; base **3043** ; repaired **2487** | 147 / 147 / 147 ; **147** | 37 / 143 / 167 ; **37** |
| 626000 | 638 / 638 / 638 ; base **1** ; repaired **637** | 472 / 472 / 472 ; **472** | 1815 / 1951 / 1951 ; base **2240** ; repaired **1817** | 225 / 225 / 225 ; **225** | 37 / 151 / 176 ; **37** |
| 734000 | 638 / 638 / 638 ; base **1** ; repaired **638** | 472 / 472 / 472 ; **472** | 1556 / 1662 / 1662 ; base **1998** ; repaired **1557** | 207 / 207 / 207 ; **207** | 37 / 144 / 168 ; **37** |
| 880000 | 639 / 639 / 639 ; base **1** ; repaired **638** | 469 / 469 / 469 ; **469** | 1840 / 1982 / 1982 ; base **2246** ; repaired **1842** | 207 / 207 / 207 ; **207** | 37 / 145 / 169 ; **37** |
| 1064000 | 638 / 638 / 638 ; base **1** ; repaired **638** | 472 / 472 / 472 ; **472** | 1550 / 1662 / 1662 ; base **2126** ; repaired **1551** | 127 / 127 / 127 ; **127** | 37 / 144 / 168 ; **37** |
| 1316000 | 638 / 638 / 638 ; base **3** ; repaired **637** | 471 / 471 / 471 ; **471** | 1792 / 1922 / 1922 ; base **2237** ; repaired **2058** | 225 / 225 / 225 ; **225** | 37 / 143 / 168 ; **37** |


Before the repair, the dialogue interval is **longer** than original by 274, 369, 289, 336, 264, 464 and 315 real frames for crystals 1–7. It starts early and waits for the recorded input; the overall scene is nevertheless shorter because the fanfare was skipped and loading/iris work is compressed. This is why the 62–233 logical boundary shift does not measure a song duration.

The loading and iris breakdown is **original H / repaired native frames**. The same compression was present before the bank repair. Properties work includes the resident song-bank change; its NMI-disabled upload is included in H.

| Seed | Properties 08/00→01 | Overlays 08/01→02 | Build 08/02→10/00 | Iris init 10/00→01 | Iris opening 10/01→09/0A | Stabilize 09/0A→09/00 |
| ---: | --- | --- | --- | --- | --- | --- |
| 380000 | 84 / 1 | 7 / 1 | 24 / 1 | 2 / 1 | 34 / 17 | 16 / 16 |
| 460000 | 84 / 1 | 7 / 1 | 24 / 1 | 2 / 1 | 34 / 17 | 16 / 16 |
| 626000 | 91 / 1 | 9 / 1 | 24 / 1 | 2 / 1 | 34 / 17 | 16 / 16 |
| 734000 | 84 / 1 | 7 / 1 | 25 / 1 | 2 / 1 | 34 / 17 | 16 / 16 |
| 880000 | 84 / 1 | 7 / 1 | 26 / 1 | 2 / 1 | 34 / 17 | 16 / 16 |
| 1064000 | 84 / 1 | 7 / 1 | 25 / 1 | 2 / 1 | 34 / 17 | 16 / 16 |
| 1316000 | 84 / 1 | 7 / 1 | 25 / 1 | 2 / 1 | 34 / 17 | 16 / 16 |

The corresponding native entry points are `overworld.rs:178` (properties), `:912` (overlays), `:6935` (build), `:2691` (iris initialization), and `:3525` (recurring iris). These are the concrete remaining phase gaps, independent of the fanfare seed error.

## Detailed phase timelines

Original cells: **logical iteration / N / H / S DSP samples**. Both native cells: **logical iteration / real frames / S DSP samples**. Every elapsed value is relative to that side’s song-issue row. `—` means that boundary/value was not observed (the old native poll already returned zero); no polygon worker runs for the pendant. Message numbers are the native/source text IDs. Thread stop occurs during the exit iris, after maiden speech ends.

### Pendant control (Eastern Palace) — seed 94000

| Boundary | Original i / N / H / S | Base i / frames / S | Repaired i / frames / S |
| --- | --- | --- | --- |
| receipt song $13 issued | 94216 / 0 / 0 / 0 | 94215 / 0 / 0 | 94215 / 0 / 0 |
| poll -> $13 | 94334 / 118 / 118 / 62908 | — | — |
| poll -> $01 | — | — | 94334 / 119 / 63546 |
| poll -> $00 | 94854 / 638 / 638 / 340131 | 94334 / 119 / 63546 | 94852 / 637 / 340158 |
| message open $059 | 94854 / 638 / 638 / 340131 | 94334 / 119 / 63546 | 94852 / 637 / 340158 |
| message close 1 | 94977 / 777 / 777 / 414235 | 94977 / 762 / 406908 | 94977 / 762 / 406908 |
| routing 13/00 | 94978 / 778 / 778 / 414768 | 94978 / 763 / 407442 | 94978 / 763 / 407442 |
| routing 13/01 | 95027 / 827 / 827 / 440891 | 95027 / 812 / 433608 | 95027 / 812 / 433608 |
| routing 13/02 | 95043 / 843 / 843 / 449421 | 95043 / 828 / 442152 | 95043 / 828 / 442152 |
| routing 13/03 | 95076 / 876 / 876 / 467014 | 95076 / 861 / 459774 | 95076 / 861 / 459774 |
| routing 13/04 | 95108 / 908 / 908 / 484074 | 95108 / 893 / 476862 | 95108 / 893 / 476862 |
| routing 13/05 | 95109 / 910 / 910 / 485141 | 95109 / 894 / 477396 | 95109 / 894 / 477396 |
| routing 08/00 | 95126 / 944 / 944 / 503267 | 95126 / 911 / 486474 | 95126 / 911 / 486474 |
| routing 08/01 | 95127 / 986 / 1009 / 537920 | 95127 / 912 / 487008 | 95127 / 912 / 487008 |
| routing 08/02 | 95128 / 991 / 1014 / 540585 | 95128 / 913 / 487542 | 95128 / 913 / 487542 |
| routing 10/00 | 95129 / 1008 / 1031 / 549648 | 95129 / 914 / 488076 | 95129 / 914 / 488076 |
| routing 10/01 | 95130 / 1010 / 1033 / 550715 | 95130 / 915 / 488610 | 95130 / 915 / 488610 |
| routing 09/0A | 95147 / 1045 / 1068 / 569374 | 95147 / 932 / 497688 | 95147 / 932 / 497688 |
| stable outdoor | 95163 / 1061 / 1084 / 577904 | 95163 / 948 / 506232 | 95163 / 948 / 506232 |
### Crystal 1 — Palace of Darkness — seed 380000

| Boundary | Original i / N / H / S | Base i / frames / S | Repaired i / frames / S |
| --- | --- | --- | --- |
| receipt song $13 issued | 380243 / 0 / 0 / 0 | 380242 / 0 / 0 | 380242 / 0 / 0 |
| poll -> $15 | 380243 / 0 / 0 / 0 | — | — |
| poll -> $13 | 380244 / 1 / 1 / 534 | — | — |
| poll -> $01 | — | — | 380243 / 1 / 534 |
| poll -> $00 | 380881 / 638 / 638 / 340132 | 380243 / 1 / 534 | 380879 / 637 / 340158 |
| routing 07/18 | 380914 / 671 / 671 / 357725 | 380276 / 34 / 18156 | 380912 / 670 / 357780 |
| polygon thread start | 380956 / 719 / 719 / 383315 | 380318 / 76 / 40584 | 380954 / 712 / 380208 |
| maiden AI 2 | 380962 / 725 / 725 / 386514 | 380324 / 82 / 43788 | 380960 / 718 / 383412 |
| maiden AI 3 | 381201 / 964 / 964 / 513930 | 380563 / 321 / 171414 | 381199 / 957 / 511038 |
| maiden AI 5 | 381418 / 1181 / 1181 / 629617 | 380780 / 538 / 287292 | 381416 / 1174 / 626916 |
| message open $132 | 381425 / 1188 / 1188 / 633349 | 380787 / 545 / 291030 | 381423 / 1181 / 630654 |
| message close 1 | 383175 / 3090 / 3090 / 1647348 | 383005 / 2763 / 1475442 | 383175 / 2933 / 1566222 |
| message open $13A | 383180 / 3095 / 3095 / 1650013 | 383010 / 2768 / 1478112 | 383180 / 2938 / 1568892 |
| message close 2 | 383339 / 3261 / 3261 / 1738511 | 383148 / 2906 / 1551804 | 383339 / 3097 / 1653798 |
| message open $139 | 383342 / 3264 / 3264 / 1740111 | 383151 / 2909 / 1553406 | 383342 / 3100 / 1655400 |
| message close 3 | 383473 / 3402 / 3402 / 1813681 | 383274 / 3032 / 1619088 | 383473 / 3231 / 1725354 |
| routing 16/00 | 383477 / 3406 / 3406 / 1815814 | 383279 / 3037 / 1621758 | 383477 / 3235 / 1727490 |
| routing 16/01 | 383534 / 3463 / 3463 / 1846202 | 383336 / 3094 / 1652196 | 383534 / 3292 / 1757928 |
| routing 16/02 | 383550 / 3479 / 3479 / 1854732 | 383352 / 3110 / 1660740 | 383550 / 3308 / 1766472 |
| routing 16/03 | 383583 / 3512 / 3512 / 1872325 | 383385 / 3143 / 1678362 | 383583 / 3341 / 1784094 |
| routing 16/04 | 383615 / 3544 / 3544 / 1889385 | 383417 / 3175 / 1695450 | 383615 / 3373 / 1801182 |
| routing 08/00 | 383630 / 3559 / 3559 / 1897382 | 383432 / 3190 / 1703460 | 383630 / 3388 / 1809192 |
| routing 08/01 | 383631 / 3619 / 3643 / 1942164 | 383433 / 3191 / 1703994 | 383631 / 3389 / 1809726 |
| routing 08/02 | 383632 / 3626 / 3650 / 1945896 | 383434 / 3192 / 1704528 | 383632 / 3390 / 1810260 |
| routing 10/00 | 383633 / 3650 / 3674 / 1958691 | 383435 / 3193 / 1705062 | 383633 / 3391 / 1810794 |
| routing 10/01 | 383634 / 3652 / 3676 / 1959757 | 383436 / 3194 / 1705596 | 383634 / 3392 / 1811328 |
| polygon thread stop | 383635 / 3654 / 3678 / 1960823 | 383437 / 3195 / 1706130 | 383635 / 3393 / 1811862 |
| routing 09/0A | 383651 / 3686 / 3710 / 1977883 | 383453 / 3211 / 1714674 | 383651 / 3409 / 1820406 |
| stable outdoor | 383667 / 3702 / 3726 / 1986413 | 383469 / 3227 / 1723218 | 383667 / 3425 / 1828950 |
### Crystal 2 — Swamp Palace — seed 460000

| Boundary | Original i / N / H / S | Base i / frames / S | Repaired i / frames / S |
| --- | --- | --- | --- |
| receipt song $13 issued | 461989 / 0 / 0 / 0 | 461988 / 0 / 0 | 461988 / 0 / 0 |
| poll -> $15 | 461989 / 0 / 0 / 0 | — | — |
| poll -> $13 | 461991 / 2 / 2 / 1066 | — | — |
| poll -> $01 | — | — | 461989 / 1 / 534 |
| poll -> $00 | 462628 / 639 / 639 / 340665 | 461989 / 1 / 534 | 462626 / 638 / 340692 |
| routing 07/18 | 462668 / 679 / 679 / 361989 | 462029 / 41 / 21894 | 462666 / 678 / 362052 |
| polygon thread start | 462710 / 727 / 727 / 387579 | 462071 / 83 / 44322 | 462708 / 720 / 384480 |
| maiden AI 2 | 462716 / 733 / 733 / 390778 | 462077 / 89 / 47526 | 462714 / 726 / 387684 |
| maiden AI 3 | 462955 / 972 / 972 / 518194 | 462316 / 328 / 175152 | 462953 / 965 / 515310 |
| maiden AI 5 | 463172 / 1189 / 1189 / 633882 | 462533 / 545 / 291030 | 463170 / 1182 / 631188 |
| message open $133 | 463179 / 1196 / 1196 / 637613 | 462540 / 552 / 294768 | 463177 / 1189 / 634926 |
| message close 1 | 465366 / 3559 / 3559 / 1897381 | 465273 / 3285 / 1754190 | 465366 / 3378 / 1803852 |
| message open $13A | 465369 / 3562 / 3562 / 1898980 | 465276 / 3288 / 1755792 | 465369 / 3381 / 1805454 |
| message close 2 | 465514 / 3714 / 3714 / 1980015 | 465435 / 3447 / 1840698 | 465514 / 3526 / 1882884 |
| message open $139 | 465521 / 3721 / 3721 / 1983747 | 465438 / 3450 / 1842300 | 465516 / 3528 / 1883952 |
| message close 3 | 465659 / 3866 / 3866 / 2061049 | 465580 / 3592 / 1918128 | 465659 / 3671 / 1960314 |
| routing 16/00 | 465663 / 3870 / 3870 / 2063182 | 465583 / 3595 / 1919730 | 465664 / 3676 / 1962984 |
| routing 16/01 | 465714 / 3921 / 3921 / 2090371 | 465634 / 3646 / 1946964 | 465715 / 3727 / 1990218 |
| routing 16/02 | 465730 / 3937 / 3937 / 2098901 | 465650 / 3662 / 1955508 | 465731 / 3743 / 1998762 |
| routing 16/03 | 465763 / 3970 / 3970 / 2116494 | 465683 / 3695 / 1973130 | 465764 / 3776 / 2016384 |
| routing 16/04 | 465795 / 4002 / 4002 / 2133554 | 465715 / 3727 / 1990218 | 465796 / 3808 / 2033472 |
| routing 08/00 | 465810 / 4017 / 4017 / 2141551 | 465730 / 3742 / 1998228 | 465811 / 3823 / 2041482 |
| routing 08/01 | 465811 / 4077 / 4101 / 2186333 | 465731 / 3743 / 1998762 | 465812 / 3824 / 2042016 |
| routing 08/02 | 465812 / 4084 / 4108 / 2190065 | 465732 / 3744 / 1999296 | 465813 / 3825 / 2042550 |
| routing 10/00 | 465813 / 4108 / 4132 / 2202860 | 465733 / 3745 / 1999830 | 465814 / 3826 / 2043084 |
| routing 10/01 | 465814 / 4110 / 4134 / 2203926 | 465734 / 3746 / 2000364 | 465815 / 3827 / 2043618 |
| polygon thread stop | 465815 / 4112 / 4136 / 2204992 | 465735 / 3747 / 2000898 | 465816 / 3828 / 2044152 |
| routing 09/0A | 465831 / 4144 / 4168 / 2222052 | 465751 / 3763 / 2009442 | 465832 / 3844 / 2052696 |
| stable outdoor | 465847 / 4160 / 4184 / 2230582 | 465767 / 3779 / 2017986 | 465848 / 3860 / 2061240 |
### Crystal 3 — Skull Woods — seed 626000

| Boundary | Original i / N / H / S | Base i / frames / S | Repaired i / frames / S |
| --- | --- | --- | --- |
| receipt song $13 issued | 626618 / 0 / 0 / 0 | 626617 / 0 / 0 | 626617 / 0 / 0 |
| poll -> $15 | 626618 / 0 / 0 / 0 | — | — |
| poll -> $13 | 626620 / 2 / 2 / 1066 | — | — |
| poll -> $01 | — | — | 626618 / 1 / 534 |
| poll -> $00 | 627256 / 638 / 638 / 340132 | 626618 / 1 / 534 | 627254 / 637 / 340158 |
| routing 07/18 | 627285 / 667 / 667 / 355592 | 626647 / 30 / 16020 | 627283 / 666 / 355644 |
| polygon thread start | 627327 / 715 / 715 / 381182 | 626689 / 72 / 38448 | 627325 / 708 / 378072 |
| maiden AI 2 | 627333 / 721 / 721 / 384381 | 626695 / 78 / 41652 | 627331 / 714 / 381276 |
| maiden AI 3 | 627575 / 963 / 963 / 513396 | 626937 / 320 / 170880 | 627573 / 956 / 510504 |
| maiden AI 5 | 627792 / 1180 / 1180 / 629084 | 627154 / 537 / 286758 | 627790 / 1173 / 626382 |
| message open $134 | 627799 / 1187 / 1187 / 632816 | 627161 / 544 / 290496 | 627797 / 1180 / 630120 |
| message close 1 | 629322 / 2832 / 2832 / 1509801 | 629104 / 2487 / 1328058 | 629322 / 2705 / 1444470 |
| message open $13A | 629324 / 2834 / 2834 / 1510868 | 629107 / 2490 / 1329660 | 629324 / 2707 / 1445538 |
| message close 2 | 629454 / 2971 / 2971 / 1583905 | 629235 / 2618 / 1398012 | 629454 / 2837 / 1514958 |
| message open $139 | 629461 / 2978 / 2978 / 1587637 | 629242 / 2625 / 1401750 | 629461 / 2844 / 1518696 |
| message close 3 | 629612 / 3136 / 3136 / 1671871 | 629398 / 2781 / 1485054 | 629612 / 2995 / 1599330 |
| routing 16/00 | 629614 / 3138 / 3138 / 1672937 | 629401 / 2784 / 1486656 | 629614 / 2997 / 1600398 |
| routing 16/01 | 629743 / 3267 / 3267 / 1741710 | 629530 / 2913 / 1555542 | 629743 / 3126 / 1669284 |
| routing 16/02 | 629759 / 3283 / 3283 / 1750240 | 629546 / 2929 / 1564086 | 629759 / 3142 / 1677828 |
| routing 16/03 | 629792 / 3316 / 3316 / 1767833 | 629579 / 2962 / 1581708 | 629792 / 3175 / 1695450 |
| routing 16/04 | 629824 / 3348 / 3348 / 1784892 | 629611 / 2994 / 1598796 | 629824 / 3207 / 1712538 |
| routing 08/00 | 629839 / 3363 / 3363 / 1792889 | 629626 / 3009 / 1606806 | 629839 / 3222 / 1720548 |
| routing 08/01 | 629840 / 3429 / 3454 / 1841403 | 629627 / 3010 / 1607340 | 629840 / 3223 / 1721082 |
| routing 08/02 | 629841 / 3438 / 3463 / 1846201 | 629628 / 3011 / 1607874 | 629841 / 3224 / 1721616 |
| routing 10/00 | 629842 / 3462 / 3487 / 1858996 | 629629 / 3012 / 1608408 | 629842 / 3225 / 1722150 |
| routing 10/01 | 629843 / 3464 / 3489 / 1860063 | 629630 / 3013 / 1608942 | 629843 / 3226 / 1722684 |
| polygon thread stop | 629844 / 3466 / 3491 / 1861129 | 629631 / 3014 / 1609476 | 629844 / 3227 / 1723218 |
| routing 09/0A | 629860 / 3498 / 3523 / 1878189 | 629647 / 3030 / 1618020 | 629860 / 3243 / 1731762 |
| stable outdoor | 629876 / 3514 / 3539 / 1886719 | 629663 / 3046 / 1626564 | 629876 / 3259 / 1740306 |
### Crystal 4 — Thieves’ Town — seed 734000

| Boundary | Original i / N / H / S | Base i / frames / S | Repaired i / frames / S |
| --- | --- | --- | --- |
| receipt song $13 issued | 735229 / 0 / 0 / 0 | 735228 / 0 / 0 | 735228 / 0 / 0 |
| poll -> $15 | 735229 / 0 / 0 / 0 | — | — |
| poll -> $13 | 735230 / 1 / 1 / 533 | — | — |
| poll -> $01 | — | — | 735229 / 1 / 534 |
| poll -> $00 | 735867 / 638 / 638 / 340132 | 735229 / 1 / 534 | 735866 / 638 / 340692 |
| routing 07/18 | 735905 / 676 / 676 / 360390 | 735267 / 39 / 20826 | 735904 / 676 / 360984 |
| polygon thread start | 735947 / 724 / 724 / 385980 | 735309 / 81 / 43254 | 735946 / 718 / 383412 |
| maiden AI 2 | 735953 / 730 / 730 / 389179 | 735315 / 87 / 46458 | 735952 / 724 / 386616 |
| maiden AI 3 | 736195 / 972 / 972 / 518194 | 735557 / 329 / 175686 | 736194 / 966 / 515844 |
| maiden AI 5 | 736412 / 1189 / 1189 / 633882 | 735774 / 546 / 291564 | 736411 / 1183 / 631722 |
| message open $135 | 736419 / 1196 / 1196 / 637614 | 735781 / 553 / 295302 | 736418 / 1190 / 635460 |
| message close 1 | 737692 / 2561 / 2561 / 1365325 | 737490 / 2262 / 1207908 | 737692 / 2464 / 1315776 |
| message open $13A | 737694 / 2563 / 2563 / 1366391 | 737492 / 2264 / 1208976 | 737694 / 2466 / 1316844 |
| message close 2 | 737828 / 2704 / 2704 / 1441562 | 737641 / 2413 / 1288542 | 737828 / 2600 / 1388400 |
| message open $139 | 737831 / 2707 / 2707 / 1443161 | 737648 / 2420 / 1292280 | 737831 / 2603 / 1390002 |
| message close 3 | 737973 / 2856 / 2856 / 1522596 | 737772 / 2544 / 1358496 | 737973 / 2745 / 1465830 |
| routing 16/00 | 737975 / 2858 / 2858 / 1523662 | 737779 / 2551 / 1362234 | 737975 / 2747 / 1466898 |
| routing 16/01 | 738086 / 2969 / 2969 / 1582839 | 737890 / 2662 / 1421508 | 738086 / 2858 / 1526172 |
| routing 16/02 | 738102 / 2985 / 2985 / 1591369 | 737906 / 2678 / 1430052 | 738102 / 2874 / 1534716 |
| routing 16/03 | 738135 / 3018 / 3018 / 1608962 | 737939 / 2711 / 1447674 | 738135 / 2907 / 1552338 |
| routing 16/04 | 738167 / 3050 / 3050 / 1626022 | 737971 / 2743 / 1464762 | 738167 / 2939 / 1569426 |
| routing 08/00 | 738182 / 3065 / 3065 / 1634019 | 737986 / 2758 / 1472772 | 738182 / 2954 / 1577436 |
| routing 08/01 | 738183 / 3125 / 3149 / 1678801 | 737987 / 2759 / 1473306 | 738183 / 2955 / 1577970 |
| routing 08/02 | 738184 / 3132 / 3156 / 1682533 | 737988 / 2760 / 1473840 | 738184 / 2956 / 1578504 |
| routing 10/00 | 738185 / 3157 / 3181 / 1695861 | 737989 / 2761 / 1474374 | 738185 / 2957 / 1579038 |
| routing 10/01 | 738186 / 3159 / 3183 / 1696927 | 737990 / 2762 / 1474908 | 738186 / 2958 / 1579572 |
| polygon thread stop | 738187 / 3161 / 3185 / 1697993 | 737991 / 2763 / 1475442 | 738187 / 2959 / 1580106 |
| routing 09/0A | 738203 / 3193 / 3217 / 1715053 | 738007 / 2779 / 1483986 | 738203 / 2975 / 1588650 |
| stable outdoor | 738219 / 3209 / 3233 / 1723583 | 738023 / 2795 / 1492530 | 738219 / 2991 / 1597194 |
### Crystal 5 — Ice Palace — seed 880000

| Boundary | Original i / N / H / S | Base i / frames / S | Repaired i / frames / S |
| --- | --- | --- | --- |
| receipt song $13 issued | 881095 / 0 / 0 / 0 | 881094 / 0 / 0 | 881094 / 0 / 0 |
| poll -> $15 | 881095 / 0 / 0 / 0 | — | — |
| poll -> $13 | 881097 / 2 / 2 / 1067 | — | — |
| poll -> $01 | — | — | 881095 / 1 / 534 |
| poll -> $00 | 881734 / 639 / 639 / 340665 | 881095 / 1 / 534 | 881732 / 638 / 340692 |
| routing 07/18 | 881763 / 668 / 668 / 356126 | 881124 / 30 / 16020 | 881761 / 667 / 356178 |
| polygon thread start | 881805 / 716 / 716 / 381716 | 881166 / 72 / 38448 | 881803 / 709 / 378606 |
| maiden AI 2 | 881811 / 722 / 722 / 384914 | 881172 / 78 / 41652 | 881809 / 715 / 381810 |
| maiden AI 3 | 882050 / 961 / 961 / 512331 | 881411 / 317 / 169278 | 882048 / 954 / 509436 |
| maiden AI 5 | 882267 / 1178 / 1178 / 628018 | 881628 / 534 / 285156 | 882265 / 1171 / 625314 |
| message open $136 | 882274 / 1185 / 1185 / 631750 | 881635 / 541 / 288894 | 882272 / 1178 / 629052 |
| message close 1 | 883836 / 2875 / 2875 / 1532726 | 883592 / 2498 / 1333932 | 883836 / 2742 / 1464228 |
| message open $13A | 883841 / 2880 / 2880 / 1535392 | 883599 / 2505 / 1337670 | 883841 / 2747 / 1466898 |
| message close 2 | 883971 / 3017 / 3017 / 1608429 | 883737 / 2643 / 1411362 | 883971 / 2877 / 1536318 |
| message open $139 | 883976 / 3022 / 3022 / 1611095 | 883739 / 2645 / 1412430 | 883976 / 2882 / 1538988 |
| message close 3 | 884110 / 3163 / 3163 / 1686265 | 883875 / 2781 / 1485054 | 884110 / 3016 / 1610544 |
| routing 16/00 | 884114 / 3167 / 3167 / 1688398 | 883881 / 2787 / 1488258 | 884114 / 3020 / 1612680 |
| routing 16/01 | 884225 / 3278 / 3278 / 1747574 | 883992 / 2898 / 1547532 | 884225 / 3131 / 1671954 |
| routing 16/02 | 884241 / 3294 / 3294 / 1756104 | 884008 / 2914 / 1556076 | 884241 / 3147 / 1680498 |
| routing 16/03 | 884274 / 3327 / 3327 / 1773697 | 884041 / 2947 / 1573698 | 884274 / 3180 / 1698120 |
| routing 16/04 | 884306 / 3359 / 3359 / 1790757 | 884073 / 2979 / 1590786 | 884306 / 3212 / 1715208 |
| routing 08/00 | 884321 / 3374 / 3374 / 1798754 | 884088 / 2994 / 1598796 | 884321 / 3227 / 1723218 |
| routing 08/01 | 884322 / 3434 / 3458 / 1843536 | 884089 / 2995 / 1599330 | 884322 / 3228 / 1723752 |
| routing 08/02 | 884323 / 3441 / 3465 / 1847268 | 884090 / 2996 / 1599864 | 884323 / 3229 / 1724286 |
| routing 10/00 | 884324 / 3467 / 3491 / 1861129 | 884091 / 2997 / 1600398 | 884324 / 3230 / 1724820 |
| routing 10/01 | 884325 / 3469 / 3493 / 1862195 | 884092 / 2998 / 1600932 | 884325 / 3231 / 1725354 |
| polygon thread stop | 884326 / 3471 / 3495 / 1863262 | 884093 / 2999 / 1601466 | 884326 / 3232 / 1725888 |
| routing 09/0A | 884342 / 3503 / 3527 / 1880322 | 884109 / 3015 / 1610010 | 884342 / 3248 / 1734432 |
| stable outdoor | 884358 / 3519 / 3543 / 1888852 | 884125 / 3031 / 1618554 | 884358 / 3264 / 1742976 |
### Crystal 6 — Misery Mire — seed 1064000

| Boundary | Original i / N / H / S | Base i / frames / S | Repaired i / frames / S |
| --- | --- | --- | --- |
| receipt song $13 issued | 1064096 / 0 / 0 / 0 | 1064095 / 0 / 0 | 1064095 / 0 / 0 |
| poll -> $15 | 1064096 / 0 / 0 / 0 | — | — |
| poll -> $13 | 1064097 / 1 / 1 / 534 | — | — |
| poll -> $01 | — | — | 1064096 / 1 / 534 |
| poll -> $00 | 1064734 / 638 / 638 / 340132 | 1064096 / 1 / 534 | 1064733 / 638 / 340692 |
| routing 07/18 | 1064766 / 670 / 670 / 357192 | 1064128 / 33 / 17622 | 1064765 / 670 / 357780 |
| polygon thread start | 1064808 / 718 / 718 / 382782 | 1064170 / 75 / 40050 | 1064807 / 712 / 380208 |
| maiden AI 2 | 1064814 / 724 / 724 / 385981 | 1064176 / 81 / 43254 | 1064813 / 718 / 383412 |
| maiden AI 3 | 1065056 / 966 / 966 / 514996 | 1064418 / 323 / 172482 | 1065055 / 960 / 512640 |
| maiden AI 5 | 1065273 / 1183 / 1183 / 630684 | 1064635 / 540 / 288360 | 1065272 / 1177 / 628518 |
| message open $137 | 1065280 / 1190 / 1190 / 634416 | 1064642 / 547 / 292098 | 1065279 / 1184 / 632256 |
| message close 1 | 1066536 / 2544 / 2544 / 1356263 | 1066473 / 2378 / 1269852 | 1066536 / 2441 / 1303494 |
| message open $13A | 1066543 / 2551 / 2551 / 1359995 | 1066475 / 2380 / 1270920 | 1066543 / 2448 / 1307232 |
| message close 2 | 1066687 / 2702 / 2702 / 1440496 | 1066617 / 2522 / 1346748 | 1066687 / 2592 / 1384128 |
| message open $139 | 1066692 / 2707 / 2707 / 1443162 | 1066620 / 2525 / 1348350 | 1066692 / 2597 / 1386798 |
| message close 3 | 1066829 / 2851 / 2851 / 1519931 | 1066761 / 2666 / 1423644 | 1066829 / 2734 / 1459956 |
| routing 16/00 | 1066830 / 2852 / 2852 / 1520464 | 1066768 / 2673 / 1427382 | 1066830 / 2735 / 1460490 |
| routing 16/01 | 1066861 / 2883 / 2883 / 1536991 | 1066799 / 2704 / 1443936 | 1066861 / 2766 / 1477044 |
| routing 16/02 | 1066877 / 2899 / 2899 / 1545521 | 1066815 / 2720 / 1452480 | 1066877 / 2782 / 1485588 |
| routing 16/03 | 1066910 / 2932 / 2932 / 1563114 | 1066848 / 2753 / 1470102 | 1066910 / 2815 / 1503210 |
| routing 16/04 | 1066942 / 2964 / 2964 / 1580174 | 1066880 / 2785 / 1487190 | 1066942 / 2847 / 1520298 |
| routing 08/00 | 1066957 / 2979 / 2979 / 1588171 | 1066895 / 2800 / 1495200 | 1066957 / 2862 / 1528308 |
| routing 08/01 | 1066958 / 3039 / 3063 / 1632953 | 1066896 / 2801 / 1495734 | 1066958 / 2863 / 1528842 |
| routing 08/02 | 1066959 / 3046 / 3070 / 1636685 | 1066897 / 2802 / 1496268 | 1066959 / 2864 / 1529376 |
| routing 10/00 | 1066960 / 3071 / 3095 / 1650013 | 1066898 / 2803 / 1496802 | 1066960 / 2865 / 1529910 |
| routing 10/01 | 1066961 / 3073 / 3097 / 1651079 | 1066899 / 2804 / 1497336 | 1066961 / 2866 / 1530444 |
| polygon thread stop | 1066962 / 3075 / 3099 / 1652146 | 1066900 / 2805 / 1497870 | 1066962 / 2867 / 1530978 |
| routing 09/0A | 1066978 / 3107 / 3131 / 1669205 | 1066916 / 2821 / 1506414 | 1066978 / 2883 / 1539522 |
| stable outdoor | 1066994 / 3123 / 3147 / 1677735 | 1066932 / 2837 / 1514958 | 1066994 / 2899 / 1548066 |
### Crystal 7 — Turtle Rock — seed 1316000

| Boundary | Original i / N / H / S | Base i / frames / S | Repaired i / frames / S |
| --- | --- | --- | --- |
| receipt song $13 issued | 1317697 / 0 / 0 / 0 | 1317696 / 0 / 0 | 1317696 / 0 / 0 |
| poll -> $15 | 1317697 / 0 / 0 / 0 | — | — |
| poll -> $13 | 1317698 / 1 / 1 / 533 | — | — |
| poll -> $01 | — | 1317697 / 1 / 534 | 1317697 / 1 / 534 |
| poll -> $00 | 1318335 / 638 / 638 / 340132 | 1317699 / 3 / 1602 | 1318333 / 637 / 340158 |
| routing 07/18 | 1318364 / 667 / 667 / 355593 | 1317728 / 32 / 17088 | 1318362 / 666 / 355644 |
| polygon thread start | 1318406 / 714 / 714 / 380649 | 1317770 / 74 / 39516 | 1318404 / 708 / 378072 |
| maiden AI 2 | 1318414 / 722 / 722 / 384914 | 1317778 / 82 / 43788 | 1318412 / 716 / 382344 |
| maiden AI 3 | 1318653 / 961 / 961 / 512330 | 1318017 / 321 / 171414 | 1318651 / 955 / 509970 |
| maiden AI 5 | 1318870 / 1178 / 1178 / 628018 | 1318234 / 538 / 287292 | 1318868 / 1172 / 625848 |
| message open $138 | 1318877 / 1185 / 1185 / 631750 | 1318241 / 545 / 291030 | 1318875 / 1179 / 629586 |
| message close 1 | 1320351 / 2775 / 2775 / 1479414 | 1320110 / 2414 / 1289076 | 1320351 / 2655 / 1417770 |
| message open $13A | 1320356 / 2780 / 2780 / 1482079 | 1320112 / 2416 / 1290144 | 1320353 / 2657 / 1418838 |
| message close 2 | 1320537 / 2968 / 2968 / 1582306 | 1320243 / 2547 / 1360098 | 1320537 / 2841 / 1517094 |
| message open $139 | 1320541 / 2972 / 2972 / 1584439 | 1320248 / 2552 / 1362768 | 1320543 / 2847 / 1520298 |
| message close 3 | 1320664 / 3102 / 3102 / 1653745 | 1320474 / 2778 / 1483452 | 1320926 / 3230 / 1724820 |
| routing 16/00 | 1320669 / 3107 / 3107 / 1656410 | 1320478 / 2782 / 1485588 | 1320933 / 3237 / 1728558 |
| routing 16/01 | 1320798 / 3236 / 3236 / 1725183 | 1320607 / 2911 / 1554474 | 1321062 / 3366 / 1797444 |
| routing 16/02 | 1320814 / 3252 / 3252 / 1733713 | 1320623 / 2927 / 1563018 | 1321078 / 3382 / 1805988 |
| routing 16/03 | 1320847 / 3285 / 3285 / 1751306 | 1320656 / 2960 / 1580640 | 1321111 / 3415 / 1823610 |
| routing 16/04 | 1320879 / 3317 / 3317 / 1768366 | 1320688 / 2992 / 1597728 | 1321143 / 3447 / 1840698 |
| routing 08/00 | 1320894 / 3332 / 3332 / 1776363 | 1320703 / 3007 / 1605738 | 1321158 / 3462 / 1848708 |
| routing 08/01 | 1320895 / 3391 / 3416 / 1821145 | 1320704 / 3008 / 1606272 | 1321159 / 3463 / 1849242 |
| routing 08/02 | 1320896 / 3398 / 3423 / 1824877 | 1320705 / 3009 / 1606806 | 1321160 / 3464 / 1849776 |
| routing 10/00 | 1320897 / 3423 / 3448 / 1838205 | 1320706 / 3010 / 1607340 | 1321161 / 3465 / 1850310 |
| routing 10/01 | 1320898 / 3425 / 3450 / 1839271 | 1320707 / 3011 / 1607874 | 1321162 / 3466 / 1850844 |
| polygon thread stop | 1320899 / 3427 / 3452 / 1840337 | 1320708 / 3012 / 1608408 | 1321163 / 3467 / 1851378 |
| routing 09/0A | 1320915 / 3459 / 3484 / 1857397 | 1320724 / 3028 / 1616952 | 1321179 / 3483 / 1859922 |
| stable outdoor | 1320931 / 3475 / 3500 / 1865927 | 1320740 / 3044 / 1625496 | 1321195 / 3499 / 1868466 |

The repaired last-message interval is 383 frames, versus 123 iterations / 130 original host frames. This input-window change explains the coincidentally equal total; earlier phases remain shorter.

## Actual APUI read positions

The cache also retains CPU-side APUI read offsets in its DSP-output window (`snes9x_compare.rs:91` and `:6680`), captured earlier at source PCs `$08:C400/$08:C609`. Reading these existing records requires no cartridge execution. Each ordinary receipt poll is in the host immediately before its first represented picture; the zero row is the last poll and the next host has none. This independently checks the display/read distinction in the phase tables.

Original cells below are **absolute host + output-sample offset within that host**. The zero cell also gives the absolute cumulative cached DSP-output sample coordinate. Native cells give the zero-poll **iteration / SPC APU-cycle coordinate before the poll** (base ; repaired). Native clocks start at the probe seed and original output samples at cache frame zero, so their absolute origins must not be compared directly. Native polls do not advance the clock: they use the SPC-derived cached is-playing value (`audio.rs:909`; `zelda_rtl/rtl_diagnostics.rs:354`). The repaired before-poll cycle is read from the retained end-of-callback summary minus that callback’s 17,088 APU cycles; the baseline coordinate follows from the same fixed callback count.

| Seed | Original $15 poll | Original $13 poll | Original zero poll ; absolute DSP sample | Native zero i / APU cycle: base ; repaired |
| ---: | --- | --- | --- | --- |
| 94000 | — | 103206 + 75 | 103726 + 75 ; 55298612 | 94334 / 5707392 ; 94852 / 14558976 |
| 380000 | 412835 + 84 | 412836 + 80 | 413473 + 85 ; 220431563 | 380243 / 4152384 ; 380879 / 15020352 |
| 460000 | 500220 + 88 | 500222 + 93 | 500859 + 88 ; 267018965 | 461989 / 33988032 ; 462626 / 44873088 |
| 626000 | 676769 + 93 | 676771 + 90 | 677407 + 93 ; 361140594 | 626618 / 10560384 ; 627254 / 21428352 |
| 734000 | 792281 + 89 | 792282 + 89 | 792919 + 94 ; 422722583 | 735229 / 21001152 ; 735866 / 31886208 |
| 880000 | 946365 + 96 | 946367 + 94 | 947004 + 90 ; 504868683 | 881095 / 18711360 ; 881732 / 29596416 |
| 1064000 | 1142854 + 94 | 1142855 + 94 | 1143492 + 87 ; 609620756 | 1064096 / 1640448 ; 1064733 / 12525504 |
| 1316000 | 1416117 + 93 | 1416118 + 94 | 1416755 + 95 ; 755303282 | 1317699 / 29032512 ; 1318333 / 39866304 |

The raw read offsets and cumulative-sample joins are retained in `cartridge-poll-offsets.json` and `cartridge-poll-times.json`. The phase tables retain the common display-boundary convention; these read rows are the actual available subframe observations.

## Stable logical outcomes

For every repaired probe, an original stable state within 2,000 iterations matches the main/submodule, room, Link X/Y, and all 1,280 bytes of `$F000..$F4FF` exactly. These are shifted-boundary outcome matches, not unshifted future-route equivalence. `target/crystal-timing/bank-outcomes.json` retains every matched boundary and SHA-256 of the full save block.

| Seed | Repaired stable | Matching original stable |
| ---: | ---: | ---: |
| 94000 | 95163 | 95163 |
| 380000 | 383667 | 383667 |
| 460000 | 465848 | 465847 |
| 626000 | 629876 | 629876 |
| 734000 | 738219 | 738219 |
| 880000 | 884358 | 884358 |
| 1064000 | 1066994 | 1066994 |
| 1316000 | 1321195 | 1320931 |

## Rupee refill tick cadence

`HudRefillPrefix` (`crates/zelda3/src/hud_refill_prefix.rs:289–302`, source `$0D:DC02..DC15`) loads the old `$0CFD`, increments the stored byte, tests the **old** low three bits, and queues SFX `$29` if they are zero. A busy sound port clears the counter. It is a refill-call counter, not an elapsed-frame timer. A lag NMI can consume the sound before this branch. No sound is authored directly by the `$0CFD` difference itself.

The probe at 162000–164000 covers room `$182` and its overlapping `$0E/02` text. The native stream predominantly requests one tick every **8 real frames** (133.114 ms), including during source lag. The cartridge’s pauses stretch the requests irregularly. The request-bearing NMI extraction uses ordered groups of four APU writes: a `$29` in group `j` belongs to the NMI accepted at `open_host + j`. The ledger lacks the store’s exact subframe bus position; using a post-iteration `picture_host` as the sound timestamp would misassign that iteration’s lag. The PCM measurements below establish the audible cadence independently.

| Consecutive native tick iterations | Native real-frame gap | Consecutive cartridge tick iterations | Cartridge request-bearing NMI hosts | Cartridge real-frame gap |
| --- | ---: | --- | --- | ---: |
| 162443 → 162451 | 8 | 162444 → 162452 | 176684 → 176707 | 23 |
| 162491 → 162499 | 8 | 162491 → 162499 | 176750 → 176775 | 25 |
| 162555 → 162563 | 8 | 162555 → 162563 | 176845 → 176872 | 27 |

Those request gaps are 382.703, 415.982 and 449.260 ms on cartridge versus 133.114 ms natively: native ticks arrive 2.875×, 3.125× and 3.375× as frequently across those pauses. These are **request-bearing NMI spacings**; final PCM onset cadence is measured separately below. Quiet, unlagged refill stretches retain the ordinary eight-frame cadence on both sides. Matching an average or adding an eight-frame timer would not reproduce the cartridge’s irregular lag-dependent pauses. Restoring the native text job’s original publication cadence would also restore the time between its refill calls; changing `$0CFD` alone does not address that cause. No rupee cadence fix was made because no small native change was established that matches those pauses without reintroducing source CPU timing or replaying recorded values.

The cadence difference is also present in **rendered PCM**, not just command queues. The immutable cache’s `presented_audio.interleaved_stereo` was extracted read-only for hosts 176675–177000; the native probe wrote its own PCM with `--audio-pcm`. The rupee sound has a distinctive ~8.3 kHz component. A fourth-order 7900–8700 Hz bandpass, 128-sample RMS envelope and rising threshold of 600 isolate its recurring onset. Repeating with threshold 800 changes the following gap measurements by less than **0.008 frame**. These are component-onset measurements of the same tick, not a claim of a sample-exact psychoacoustic first-audible threshold.

| Tick pair above | Native PCM gap, real frames | Cartridge PCM gap, real frames | Native − cartridge |
| --- | ---: | ---: | ---: |
| First pair | 8.127 | 23.692 | −15.565 f (−259 ms) |
| Second pair | 7.682 | 25.195 | −17.513 f (−291 ms) |
| Third pair | 8.184 | 26.777 | −18.593 f (−309 ms) |

This is a plainly audible cadence change during those pauses. SPC polling and the tick’s envelope account for the subframe difference from the integer request-NMI gaps. Native frame coordinates use 534 samples per frame; original coordinates use each cached host’s actual 533/534-sample output count. The retained evidence is `cartridge-rupee-audio.jsonl`, `lagless-rupee.pcm`, `rupee-pcm-onsets.json` and `rupee-spectrogram.png`. No cartridge execution or cached-A/V replay was used to obtain it.

## Verification and reproducibility

No commit/push, `cargo fmt`, GPU/video run, Snes9x run or cached-A/V execution was performed. The ledger and cache were opened read-only. The separate `zelda3-rs-codex` worktree was not accessed. Scratch analysis programs live under `/tmp`, not in the repository.

The baseline was recorded from the clean `6dc983f2` default binary before the native change. Its metadata records `git_dirty: false`, binary SHA-256 `502623f05dda0d9e820fe31951dc4e352165b596f60b1642e0cc0d0614308784`, and ledger SHA-256 `18b58a3f887457d770918497218eaf7eb39360e1d04ed789c8b04b0ff4d8ba77`.

| Check | Result |
| --- | --- |
| Reduced binary build | PASS, no warnings (`bank-build.log`; final harness build `final-reduced-build.log`) |
| Default binary build | PASS, no warnings (`final-default-build.log`; also `bank-default-build.log`) |
| Default mode vs clean baseline | PASS: all 200,000 records and EOF (`final-default-compare.log`; also passed earlier in `default-compare.log`) |
| Default Zelda + SNES lib suites | Zelda: **2454 passed, 1 permitted spotlight failure, 3 ignored** (exit 101). SNES: **504 passed, 0 failed, 7 ignored**. 24 existing default fixture warnings; ordinary binary builds have none. |
| Reduced Zelda + SNES lib suites | **1989 Zelda + 504 SNES passed**, 0 failed; 2 + 7 ignored (exit 0), no warnings |
| Full cartridge-answer census / gate | **PASS**, exit 0 / gate 0: 1,473,123 iterations, 739 segments accepted under the existing classifications, unexplained 0 (`census-rom.log`, `census-rom-summary.txt`) |
| Full live-SPC census / gate | **PASS**, exit 0 / gate 0: 1,473,123 iterations, 742 segments accepted under the existing classifications, unexplained 0 (`census-live.log`, `census-live-summary.txt`) |
| All eight uninterrupted outcome probes | PASS: full save and logical outcome at the stable boundary |
| Complete native on-screen phase-duration target | **NOT MET**; residual phase differences quantified above |

The sole default failure is `cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`, the permitted existing failure.

Full censuses use retained `bank-reduced-zelda3`. The later `final-reduced-zelda3` adds optional PCM output and window argument validation to the harness; neither is active in a full census. Its build reused the unchanged native Zelda crate. Both ordinary feature builds are retained, and `final-default-zelda3` is the binary used for the final 200k baseline comparison. Binary and evidence hashes are in `artifact-hashes.json`; `engine-change.patch` binds the native repair separately from `harness-change.patch`.

The census gates retain the existing polygon, song-poll, and rupee exceptions. A passing gate establishes their existing logical-state coverage/classification contract, not equal on-screen duration.

All logs and raw timing traces are under `target/crystal-timing/`. `before.jsonl` is the old engine, `bank.jsonl` is the seed-bank repair; each retains both engine and original state observations. Their `.phases.json` files retain the extracted boundaries, including absolute original host values. `bank.jsonl` additionally retains the actual native SPC summaries. These target artifacts are local evidence, not a promotion or production acceptance receipt.

```sh
LEDGER=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
BIN=target/crystal-timing/final-reduced-zelda3
ZELDA3_SAVE_DIR=/tmp/crystal-receipt-repro "$BIN" --logical-replay "$LEDGER" "$C/initial.srm" \
  --seed-every 2000 --single-seed --live-apu-reads --report 0 \
  --windows 94000-96000,380000-384500,460000-466500,626000-630500,734000-739000,880000-885000,1064000-1068000,1316000-1322500 \
  --outcome-trace /tmp/crystal-receipt-repro.jsonl
```
