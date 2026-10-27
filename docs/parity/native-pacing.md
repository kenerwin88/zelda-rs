# Native presentation pacing

## Phase 1: route lag inventory

The read-only ledger contains **1,473,123 iterations, 39,575 iterations with lag, and 102,677 accepted lag NMIs**. The table is sorted by lag NMIs; the final column is the cumulative share. Iterations includes zero-lag calls in the same context. Source routine identifies the dispatcher owner; the ledger has no main-thread PC trace, so it cannot assign each individual overrun to a child routine.

WRAM delta image `j` is the opening boundary for iteration `j`; image `j+1` is its result. This inventory joins `iterations[j].lag_nmis` to image `j`, including the initial image. Context extraction streamed all 1,473,124 images read-only. The scratch analysis initially used the previous image; that error was corrected before this report. `target/native-pacing/inventory.json`, `contexts.bin` and `transitions.json` retain the corrected data. No runtime code reads them.

Scene work is deterministic given its state and operands; gameplay load includes ordinary sprite-heavy/control calls. A dispatcher label does not prove which child consumed the budget. In particular text includes its sprite/HUD prefix, and terminal iris callers can include transition tails. The source ledger alone cannot split these costs further.

| Entry module/submodule | Source routine / owner | Work kind | Iterations | Lagged iterations | Lag NMIs | Share | Cumulative |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | RenderText (VWF, message scroll) | Scene: text / scroll | 58,097 | 4,418 | 12,964 | 12.63% | 12.63% |
| `07/02` | module07_02_supertile_transition | Scene: transition / UI / cutscene | 33,311 | 1,976 | 8,770 | 8.54% | 21.17% |
| `06/00` | module_pre_dungeon | Scene: load / upload | 142 | 142 | 8,196 | 7.98% | 29.15% |
| `07/0E` | module07_0e_spiral_stairs | Scene: transition / UI / cutscene | 27,788 | 4,952 | 7,979 | 7.77% | 36.92% |
| `08/00` | pre_overworld_load_properties (incl. song bank) | Scene: load / upload | 157 | 157 | 6,466 | 6.30% | 43.22% |
| `09/23` | module09_mirror_warp | Scene: load / upload | 12,279 | 1,179 | 6,054 | 5.90% | 49.11% |
| `0F/01` | module0f_spotlight_close | Scene: iris | 4,879 | 4,879 | 5,432 | 5.29% | 54.40% |
| `09/03` | module09_load_new_map_and_gfx | Scene: load / upload | 251 | 251 | 4,372 | 4.26% | 58.66% |
| `07/07` | module07_07_falling_transition | Scene: transition / UI / cutscene | 5,591 | 2,919 | 4,060 | 3.95% | 62.62% |
| `07/00` | module07_00_player_control | Gameplay load | 780,760 | 2,960 | 3,679 | 3.58% | 66.20% |
| `09/01` | module09_load_aux_gfx | Scene: load / upload | 251 | 251 | 3,479 | 3.39% | 69.59% |
| `07/0F` | module07_0f_landing_wipe | Scene: iris | 2,718 | 2,718 | 2,992 | 2.91% | 72.50% |
| `10/01` | module10_spotlight_open | Scene: iris | 2,669 | 2,669 | 2,672 | 2.60% | 75.10% |
| `0E/03` | module0e_03_dungeon_map | Scene: transition / UI / cutscene | 13,047 | 115 | 2,672 | 2.60% | 77.71% |
| `08/02` | overworld_load_and_build_screen | Scene: load / upload | 157 | 157 | 2,576 | 2.51% | 80.22% |
| `07/15` | module07_15_warp_pad | Scene: transition / UI / cutscene | 2,900 | 1,735 | 2,333 | 2.27% | 82.49% |
| `19/02` | module19_triforce_room | Scene: polygon / cutscene | 3,559 | 1,855 | 2,001 | 1.95% | 84.44% |
| `0E/0A` | module0e_0a_flute_menu | Scene: transition / UI / cutscene | 10,081 | 90 | 1,845 | 1.80% | 86.23% |
| `09/00` | module09_00_player_control | Gameplay load | 303,167 | 1,192 | 1,253 | 1.22% | 87.45% |
| `0E/07` | messaging_overworld_map | Scene: transition / UI / cutscene | 4,441 | 56 | 1,065 | 1.04% | 88.49% |
| `07/06` | module07_06_fat_inter_room_stairs | Scene: transition / UI / cutscene | 1,360 | 603 | 984 | 0.96% | 89.45% |
| `11/00` | module11_dungeon_falling_entrance | Scene: load / upload | 1,079 | 17 | 973 | 0.95% | 90.40% |
| `07/01` | module07_01_subtile_transition | Scene: transition / UI / cutscene | 32,647 | 928 | 928 | 0.90% | 91.30% |
| `08/01` | pre_overworld_load_overlays | Scene: load / upload | 157 | 157 | 706 | 0.69% | 91.99% |
| `05/00` | module05_load_file | Scene: transition / UI / cutscene | 11 | 11 | 639 | 0.62% | 92.61% |
| `09/04` | module09_load_new_sprites | Scene: load / upload | 251 | 251 | 508 | 0.49% | 93.11% |
| `12/03` | module12_game_over | Scene: transition / UI / cutscene | 391 | 391 | 472 | 0.46% | 93.57% |
| `0F/00` | module0f_spotlight_close | Scene: iris | 287 | 287 | 449 | 0.44% | 94.00% |
| `07/12` | module07_11_straight_interroom_stairs | Scene: transition / UI / cutscene | 1,026 | 239 | 391 | 0.38% | 94.38% |
| `09/2E` | module09_2e_whirlpool | Scene: load / upload | 895 | 100 | 363 | 0.35% | 94.74% |
| `07/16` | module07_16_update_pegs | Scene: transition / UI / cutscene | 1,616 | 101 | 303 | 0.30% | 95.03% |
| `09/11` | module09_load_new_map_and_gfx | Scene: load / upload | 16 | 16 | 274 | 0.27% | 95.30% |
| `09/16` | module09_fade_back_in_from_mosaic | Scene: transition / UI / cutscene | 528 | 233 | 265 | 0.26% | 95.56% |
| `09/0D` | overworld_start_mosaic_transition | Scene: transition / UI / cutscene | 528 | 223 | 247 | 0.24% | 95.80% |
| `0E/0B` | save_menu | Scene: transition / UI / cutscene | 1,351 | 81 | 243 | 0.24% | 96.03% |
| `09/0F` | module09_load_aux_gfx | Scene: load / upload | 16 | 16 | 242 | 0.24% | 96.27% |
| `01/00` | module01_file_select | Scene: transition / UI / cutscene | 4 | 4 | 235 | 0.23% | 96.50% |
| `09/2C` | module09_mirror_warp | Scene: load / upload | 477 | 36 | 228 | 0.22% | 96.72% |
| `10/00` | module10_spotlight_open | Scene: iris | 157 | 157 | 157 | 0.15% | 96.87% |
| `17/01` | module17_save_and_quit | Scene: transition / UI / cutscene | 45 | 3 | 141 | 0.14% | 97.01% |
| `1B/02` | module1b_spawn_select | Scene: transition / UI / cutscene | 263 | 9 | 139 | 0.14% | 97.15% |
| `09/20` | overworld_load_overlays2 | Scene: load / upload | 28 | 28 | 138 | 0.13% | 97.28% |
| `0E/05` | module0e_05_desert_prayer | Scene: transition / UI / cutscene | 122 | 119 | 138 | 0.13% | 97.42% |
| `00/01` | module00_intro | Scene: polygon / cutscene | 25 | 11 | 114 | 0.11% | 97.53% |
| `0B/24` | overworld_start_mosaic_transition | Scene: transition / UI / cutscene | 132 | 93 | 103 | 0.10% | 97.63% |
| `09/21` | overworld_load_ambient_overlay_false | Scene: transition / UI / cutscene | 28 | 28 | 84 | 0.08% | 97.71% |
| `09/0E` | pre_overworld_load_overlays | Scene: load / upload | 16 | 16 | 83 | 0.08% | 97.79% |
| `15/07` | module15_mirror_warp_from_aga | Scene: transition / UI / cutscene | 2,843 | 79 | 83 | 0.08% | 97.87% |
| `19/00` | module19_triforce_room | Scene: polygon / cutscene | 358 | 4 | 72 | 0.07% | 97.94% |
| `07/13` | module07_11_straight_interroom_stairs | Scene: transition / UI / cutscene | 166 | 47 | 71 | 0.07% | 98.01% |
| `0B/17` | overworld_start_mosaic_transition | Scene: transition / UI / cutscene | 132 | 65 | 68 | 0.07% | 98.08% |
| `0B/28` | overworld_load_and_build_screen | Scene: load / upload | 4 | 4 | 68 | 0.07% | 98.14% |
| `0B/2B` | overworld_func2b | Scene: transition / UI / cutscene | 67 | 66 | 66 | 0.06% | 98.21% |
| `0B/19` | overworld_func19 | Scene: transition / UI / cutscene | 4 | 4 | 64 | 0.06% | 98.27% |
| `15/03` | module15_mirror_warp_from_aga | Scene: transition / UI / cutscene | 114 | 12 | 64 | 0.06% | 98.33% |
| `00/05` | module00_intro | Scene: polygon / cutscene | 123 | 62 | 62 | 0.06% | 98.39% |
| `0B/26` | module09_load_aux_gfx | Scene: load / upload | 4 | 4 | 61 | 0.06% | 98.45% |
| `0B/1A` | module09_load_aux_gfx | Scene: transition / UI / cutscene | 4 | 4 | 60 | 0.06% | 98.51% |
| `1A/12` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 60 | 0.06% | 98.57% |
| `0B/29` | module09_fade_back_in_from_mosaic | Scene: transition / UI / cutscene | 132 | 51 | 59 | 0.06% | 98.62% |
| `18/00` | module18_ganon_emerges | Scene: transition / UI / cutscene | 931 | 2 | 59 | 0.06% | 98.68% |
| `1A/00` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 59 | 0.06% | 98.74% |
| `1A/1A` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 59 | 0.06% | 98.80% |
| `1A/1C` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 59 | 0.06% | 98.85% |
| `1A/04` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 58 | 0.06% | 98.91% |
| `1A/08` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 58 | 0.06% | 98.97% |
| `1A/0A` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 58 | 0.06% | 99.02% |
| `1A/18` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 58 | 0.06% | 99.08% |
| `1A/0E` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 57 | 0.06% | 99.14% |
| `1A/10` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 56 | 0.05% | 99.19% |
| `1A/06` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 55 | 0.05% | 99.24% |
| `07/03` | module07_03_overlay_change | Scene: transition / UI / cutscene | 67 | 53 | 53 | 0.05% | 99.30% |
| `1A/0C` | module1_a_credits | Scene: polygon / cutscene | 3 | 2 | 53 | 0.05% | 99.35% |
| `11/07` | module11_dungeon_falling_entrance | Scene: load / upload | 701 | 17 | 51 | 0.05% | 99.40% |
| `13/05` | module13_boss_victory_pendant | Scene: transition / UI / cutscene | 51 | 51 | 51 | 0.05% | 99.45% |
| `1A/1E` | module1_a_credits | Scene: polygon / cutscene | 3 | 2 | 51 | 0.05% | 99.50% |
| `0B/1C` | overworld_func1c | Scene: transition / UI / cutscene | 132 | 40 | 48 | 0.05% | 99.54% |
| `1A/14` | module1_a_credits | Scene: polygon / cutscene | 1 | 1 | 47 | 0.05% | 99.59% |
| `1A/02` | module1_a_credits | Scene: polygon / cutscene | 1 | 1 | 46 | 0.04% | 99.63% |
| `1A/16` | module1_a_credits | Scene: polygon / cutscene | 1 | 1 | 45 | 0.04% | 99.68% |
| `07/18` | module07_18_rescued_maiden | Scene: transition / UI / cutscene | 294 | 14 | 41 | 0.04% | 99.72% |
| `1A/20` | module1_a_credits | Scene: polygon / cutscene | 1 | 1 | 41 | 0.04% | 99.76% |
| `09/12` | module09_load_new_sprites | Scene: load / upload | 16 | 16 | 32 | 0.03% | 99.79% |
| `12/08` | module12_game_over | Scene: transition / UI / cutscene | 4 | 4 | 32 | 0.03% | 99.82% |
| `0B/25` | overworld_load_overlays | Scene: load / upload | 4 | 4 | 30 | 0.03% | 99.85% |
| `0B/00` | module09_00_player_control | Gameplay load | 12,309 | 13 | 29 | 0.03% | 99.88% |
| `0B/18` | overworld_func18 | Scene: transition / UI / cutscene | 4 | 4 | 24 | 0.02% | 99.90% |
| `12/02` | module12_game_over | Scene: transition / UI / cutscene | 736 | 23 | 23 | 0.02% | 99.92% |
| `12/05` | module12_game_over | Scene: transition / UI / cutscene | 2,208 | 19 | 19 | 0.02% | 99.94% |
| `00/07` | module00_intro | Scene: polygon / cutscene | 175 | 14 | 14 | 0.01% | 99.96% |
| `15/05` | module15_mirror_warp_from_aga | Scene: transition / UI / cutscene | 1 | 1 | 14 | 0.01% | 99.97% |
| `01/02` | module01_file_select | Scene: transition / UI / cutscene | 7 | 7 | 7 | 0.01% | 99.98% |
| `00/0A` | module00_intro | Scene: polygon / cutscene | 3 | 3 | 6 | 0.01% | 99.98% |
| `12/09` | module12_game_over | Scene: transition / UI / cutscene | 720 | 4 | 4 | 0.00% | 99.99% |
| `13/04` | module13_boss_victory_pendant | Scene: transition / UI / cutscene | 3 | 3 | 3 | 0.00% | 99.99% |
| `00/02` | module00_intro | Scene: polygon / cutscene | 1 | 1 | 2 | 0.00% | 99.99% |
| `00/06` | module00_intro | Scene: polygon / cutscene | 84 | 2 | 2 | 0.00% | 99.99% |
| `04/01` | module04_name_file | Scene: transition / UI / cutscene | 2 | 2 | 2 | 0.00% | 99.99% |
| `00/00` | module00_intro | Scene: polygon / cutscene | 1 | 1 | 1 | 0.00% | 100.00% |
| `03/01` | module03_kill_file | Scene: transition / UI / cutscene | 1 | 1 | 1 | 0.00% | 100.00% |
| `0E/01` | hud_module_run | Scene: transition / UI / cutscene | 69,382 | 1 | 1 | 0.00% | 100.00% |
| `09/06` | overworld_run_scroll_transition | Scene: transition / UI / cutscene | 7,556 | 1 | 1 | 0.00% | 100.00% |
| `07/0B` | module07_0b_drain_swamp_pool | Scene: transition / UI / cutscene | 205 | 1 | 1 | 0.00% | 100.00% |
| `19/01` | module19_triforce_room | Scene: polygon / cutscene | 274 | 1 | 1 | 0.00% | 100.00% |

Ordinary gameplay-control contexts (`07/00`, `09/00`, `0B/00`) total **4,961 lag NMIs (4.83%)**; the other scene/transition/UI contexts total **97,716 (95.17%)**. The polygon-worker-active view overlaps these categories: **27,104 iterations, 2,408 lagged iterations, 3,939 lag NMIs**. It is not added to the disjoint table.

Contexts with no lag are omitted from the table but included in the route total and equality denominator. Before this change, native held frames were zero for every iteration: equality therefore means exactly the zero-lag iterations, not equivalent pacing.

NMI counts exclude elapsed time while NMI is disabled. Crystal song-bank changes include roughly 24 additional presentation frames not represented by `lag_nmis`; these require a separate SPC-upload elapsed-time model. They must not be counted as accepted NMI-only holds.

## Scope and implementation

This is a partial implementation: native NMI-only callbacks plus coarse iris work pacing. Loading budgets, VWF text/scroll, and full elapsed polygon pacing remain open. The follow-up below adds song-upload elapsed time separately from accepted NMI holds. The complete user pacing target is not met.

`rtl_native_pacing.rs` retains a serde-skipped pending-hold count. A held callback runs the normal NMI and APU publication, retaining the software update latch, and skips main, state recording, input advancement and RNG-frame advancement. Reset and foreign-memory seeding clear the pending work. Default and ROM-timing execution do not accrue holds.

The iris budget comes from actual iterations of `$00:F361..F3A0`: one work unit per row pair and two extra units when `$00:F4CC..F53D` performs circle math. One unit represents half a scanline of coarse throughput. The projection stage respects the `$00:F3A3..F3B2` wait for line 192, budgets 28 lines for the 224-word programmed copy `$00:F3B7..F3C3`, and eight lines for the radius/control/caller tail. Entry is approximated at line 250. These estimates are independent of scenes, crystals, rooms, ledger counts and cycle-ledger prices. They are approximate work costs, not instruction timing.

The completed table is still published atomically before the held callbacks. Partial table generation/publication and source caller suspension are **not** restored. Existing polygon lag phases already execute at the ordinary closing NMI; these held callbacks do not replay those phases. This patch does not establish elapsed polygon pacing.

The CPU replay harness drains pending held callbacks, rendering one APU frame per callback and observing every callback's command ports. `logical-segment-pacing` and `logical-pacing-category` report exact-count equality, native held frames and ledger lag NMIs. `--outcome-trace` adds a `native_held` column per iteration. The existing logical-state gate reports pacing separately; partial pacing mismatches are not hidden as a PASS on duration. If pacing rows are present, the gate also rejects missing/duplicate segment rows and impossible equality counts. Its 20k valid probe passes; removing a pacing row or injecting an impossible equality count fails (`pacing-gate-checks.json`).

The `$0CFD` exception can now begin only when native held frames differ from the source lag count, and may carry only until that byte reconverges. A newly matched-count iteration cannot start the exception merely because the source lagged. Matching a total count does not prove correct mid-iteration NMI placement; the byte still has to compare cleanly. The four existing map/flute post-NMI sampling signatures retain their separate exception. Text pacing is unimplemented, so this patch is not a rupee refill cadence repair.

## Verification and remaining work

The 200,000-frame default-mode baseline was recorded from clean main `b5dde888` before edits, using `main-default-zelda3`. `final-default-zelda3` matches every state, display, SRAM, audio, input and context record plus EOF (`default-compare.log`). Both ordinary binary feature builds are warning-free.

### Crystal loading and iris phases

Each cell is **original presentation-host frames / before / after native frames**. Before is main's seed-bank repair (`bank.jsonl.phases.json`); after uses the same uninterrupted live-SPC windows, with one native presentation frame per main or held callback. The pendant is a control. Original H includes the NMI-disabled song upload.

| Seed | Properties 08/00→01 | Overlays 08/01→02 | Build 08/02→10/00 | Iris init 10/00→01 | Iris opening 10/01→09/0A | Stabilize 09/0A→09/00 |
| ---: | --- | --- | --- | --- | --- | --- |

| 94000 | 65 / 1 / 1 | 5 / 1 / 1 | 17 / 1 / 1 | 2 / 1 / 2 | 35 / 17 / 34 | 16 / 16 / 16 |
| 380000 | 84 / 1 / 1 | 7 / 1 / 1 | 24 / 1 / 1 | 2 / 1 / 2 | 34 / 17 / 34 | 16 / 16 / 16 |
| 460000 | 84 / 1 / 1 | 7 / 1 / 1 | 24 / 1 / 1 | 2 / 1 / 2 | 34 / 17 / 34 | 16 / 16 / 16 |
| 626000 | 91 / 1 / 1 | 9 / 1 / 1 | 24 / 1 / 1 | 2 / 1 / 2 | 34 / 17 / 34 | 16 / 16 / 16 |
| 734000 | 84 / 1 / 1 | 7 / 1 / 1 | 25 / 1 / 1 | 2 / 1 / 2 | 34 / 17 / 34 | 16 / 16 / 16 |
| 880000 | 84 / 1 / 1 | 7 / 1 / 1 | 26 / 1 / 1 | 2 / 1 / 2 | 34 / 17 / 34 | 16 / 16 / 16 |
| 1064000 | 84 / 1 / 1 | 7 / 1 / 1 | 25 / 1 / 1 | 2 / 1 / 2 | 34 / 17 / 34 | 16 / 16 / 16 |
| 1316000 | 84 / 1 / 1 | 7 / 1 / 1 | 25 / 1 / 1 | 2 / 1 / 2 | 34 / 17 / 34 | 16 / 16 / 16 |

Iris initialization and opening now meet the 1–2-frame target in all eight probes (exact for all seven crystals; the pendant opening is one frame short). The three loading phases remain compressed. No claim of complete scene-duration parity is made.

All eight probes retain the same stable main/submodule, room, Link X/Y and every byte of `$F000..$F4FF` as an original stable boundary within 2,000 iterations. Native stable boundaries remain 95163, 383667, 465848, 629876, 738219, 884358, 1066994 and 1321195. Corresponding original boundaries are 95163, 383667, 465847, 629876, 738219, 884358, 1066994 and 1320931. These are shifted logical outcomes, not future-route or video acceptance. `crystals.jsonl` retains every raw per-iteration hold count, state and SPC summary; `crystal-phases.json` retains boundary times and complete-save hashes.

### Complete receipt phase groups

Cells are original H / before / after, in real frames. Message duration includes recorded player dismissal waits. This is the same grouping as `crystal-receipt-timing.md`, with main `b5dde888` as before.

| Seed | Song → zero | Thread → speech | Messages → victory | Victory → load | Load → stable |
| ---: | --- | --- | --- | --- | --- |
| 94000 | 638 / 637 / 637 | — | — | — | 140 / 37 / 55 |
| 380000 | 638 / 637 / 637 | 469 / 469 / 469 | 2218 / 2054 / 2054 | 153 / 153 / 153 | 167 / 37 / 55 |
| 460000 | 639 / 638 / 638 | 469 / 469 / 469 | 2674 / 2487 / 2487 | 147 / 147 / 147 | 167 / 37 / 55 |
| 626000 | 638 / 637 / 637 | 472 / 472 / 472 | 1951 / 1817 / 1817 | 225 / 225 / 225 | 176 / 37 / 55 |
| 734000 | 638 / 638 / 638 | 472 / 472 / 472 | 1662 / 1557 / 1557 | 207 / 207 / 207 | 168 / 37 / 55 |
| 880000 | 639 / 638 / 638 | 469 / 469 / 469 | 1982 / 1842 / 1842 | 207 / 207 / 207 | 169 / 37 / 55 |
| 1064000 | 638 / 638 / 638 | 472 / 472 / 472 | 1662 / 1551 / 1551 | 127 / 127 / 127 | 168 / 37 / 55 |
| 1316000 | 638 / 637 / 637 | 471 / 471 / 471 | 1922 / 2058 / 2058 | 225 / 225 / 225 | 168 / 37 / 55 |


### Full-route pacing metrics

Before holds were zero by construction; its equality counts are the ledger's zero-lag calls. This is why the before equality rate is already 97.31% despite losing every lag frame. After counts are measured independently by the native callback drain. Every category's iteration count and lag sum was checked against the read-only Phase 1 inventory.

| Mode | Exact-count iterations before | Exact-count iterations after | Held frames before → after | Ledger lag NMIs |
| --- | ---: | ---: | ---: | ---: |
| Cartridge APU answers | 1,433,548 / 1,473,123 | 1,443,663 / 1,473,123 | 0 → 11,178 | 102,677 |
| Live SPC answers | 1,433,548 / 1,473,123 | 1,443,658 / 1,473,123 | 0 → 11,176 | 102,677 |

Cartridge-answer equality improves from **97.3135% to 98.0002%**. Only **10.89% of total lag frames** have been restored; most remaining lag is outside this iris batch. The count rate alone must not be presented as complete pacing parity. For the five primary iris contexts (`0F/00..01`, `07/0F`, `10/00..01`), native holds total **10,710 versus 11,702** source lag NMIs. Shared game-over and pendant iris callers account for the other 468 native held frames.

In the table below, equality is **before / cartridge-answer after / live-SPC after**. Held frames are **cartridge / live**, with zero before in every row. Contexts with no source lag and no native holds are omitted; they remain in overall totals and match all zero-lag counts.

| Entry context | Iterations | Equal-count iterations before / cartridge / live | Native held frames cartridge / live | Ledger lag NMIs |
| --- | ---: | ---: | ---: | ---: |
| `0E/02` | 58,097 | 53,679 / 53,679 / 53,679 | 0 / 0 | 12,964 |
| `07/02` | 33,311 | 31,335 / 31,335 / 31,335 | 0 / 0 | 8,770 |
| `06/00` | 142 | 0 / 0 / 0 | 0 / 0 | 8,196 |
| `07/0E` | 27,788 | 22,836 / 22,836 / 22,836 | 0 / 0 | 7,979 |
| `08/00` | 157 | 0 / 0 / 0 | 0 / 1 | 6,466 |
| `09/23` | 12,279 | 11,100 / 11,100 / 11,100 | 0 / 0 | 6,054 |
| `0F/01` | 4,879 | 0 / 4,326 / 4,326 | 4,879 / 4,879 | 5,432 |
| `09/03` | 251 | 0 / 0 / 0 | 0 / 0 | 4,372 |
| `07/07` | 5,591 | 2,672 / 2,672 / 2,672 | 0 / 0 | 4,060 |
| `07/00` | 780,760 | 777,800 / 777,800 / 777,800 | 0 / 0 | 3,679 |
| `09/01` | 251 | 0 / 0 / 0 | 0 / 0 | 3,479 |
| `07/0F` | 2,718 | 0 / 2,454 / 2,454 | 2,718 / 2,718 | 2,992 |
| `0E/03` | 13,047 | 12,932 / 12,932 / 12,932 | 0 / 0 | 2,672 |
| `10/01` | 2,669 | 0 / 2,666 / 2,664 | 2,669 / 2,667 | 2,672 |
| `08/02` | 157 | 0 / 0 / 0 | 0 / 1 | 2,576 |
| `07/15` | 2,900 | 1,165 / 1,165 / 1,165 | 0 / 0 | 2,333 |
| `19/02` | 3,559 | 1,704 / 1,704 / 1,704 | 0 / 0 | 2,001 |
| `0E/0A` | 10,081 | 9,991 / 9,991 / 9,991 | 0 / 0 | 1,845 |
| `09/00` | 303,167 | 301,975 / 301,975 / 301,975 | 0 / 0 | 1,253 |
| `0E/07` | 4,441 | 4,385 / 4,385 / 4,385 | 0 / 0 | 1,065 |
| `07/06` | 1,360 | 757 / 757 / 757 | 0 / 0 | 984 |
| `11/00` | 1,079 | 1,062 / 1,062 / 1,062 | 0 / 0 | 973 |
| `07/01` | 32,647 | 31,719 / 31,719 / 31,719 | 0 / 0 | 928 |
| `08/01` | 157 | 0 / 0 / 0 | 0 / 1 | 706 |
| `05/00` | 11 | 0 / 0 / 0 | 0 / 0 | 639 |
| `09/04` | 251 | 0 / 0 / 0 | 0 / 0 | 508 |
| `12/03` | 391 | 0 / 310 / 310 | 391 / 391 | 472 |
| `0F/00` | 287 | 0 / 125 / 125 | 287 / 287 | 449 |
| `07/12` | 1,026 | 787 / 787 / 787 | 0 / 0 | 391 |
| `09/2E` | 895 | 795 / 795 / 795 | 0 / 0 | 363 |
| `07/16` | 1,616 | 1,515 / 1,515 / 1,515 | 0 / 0 | 303 |
| `09/11` | 16 | 0 / 0 / 0 | 0 / 0 | 274 |
| `09/16` | 528 | 295 / 295 / 295 | 0 / 0 | 265 |
| `09/0D` | 528 | 305 / 305 / 305 | 0 / 0 | 247 |
| `0E/0B` | 1,351 | 1,270 / 1,270 / 1,270 | 0 / 0 | 243 |
| `09/0F` | 16 | 0 / 0 / 0 | 0 / 0 | 242 |
| `01/00` | 4 | 0 / 0 / 0 | 0 / 0 | 235 |
| `09/2C` | 477 | 441 / 441 / 441 | 0 / 0 | 228 |
| `10/00` | 157 | 0 / 157 / 154 | 157 / 154 | 157 |
| `17/01` | 45 | 42 / 42 / 42 | 0 / 0 | 141 |
| `1B/02` | 263 | 254 / 254 / 254 | 0 / 0 | 139 |
| `09/20` | 28 | 0 / 0 / 0 | 0 / 0 | 138 |
| `0E/05` | 122 | 3 / 3 / 3 | 0 / 0 | 138 |
| `00/01` | 25 | 14 / 14 / 14 | 0 / 0 | 114 |
| `0B/24` | 132 | 39 / 39 / 39 | 0 / 0 | 103 |
| `09/21` | 28 | 0 / 0 / 0 | 0 / 0 | 84 |
| `09/0E` | 16 | 0 / 0 / 0 | 0 / 0 | 83 |
| `15/07` | 2,843 | 2,764 / 2,764 / 2,764 | 0 / 0 | 83 |
| `19/00` | 358 | 354 / 354 / 354 | 0 / 0 | 72 |
| `07/13` | 166 | 119 / 119 / 119 | 0 / 0 | 71 |
| `0B/17` | 132 | 67 / 67 / 67 | 0 / 0 | 68 |
| `0B/28` | 4 | 0 / 0 / 0 | 0 / 0 | 68 |
| `0B/2B` | 67 | 1 / 1 / 1 | 0 / 0 | 66 |
| `0B/19` | 4 | 0 / 0 / 0 | 0 / 0 | 64 |
| `15/03` | 114 | 102 / 102 / 102 | 0 / 0 | 64 |
| `00/05` | 123 | 61 / 61 / 61 | 0 / 0 | 62 |
| `0B/26` | 4 | 0 / 0 / 0 | 0 / 0 | 61 |
| `0B/1A` | 4 | 0 / 0 / 0 | 0 / 0 | 60 |
| `1A/12` | 3 | 0 / 0 / 0 | 0 / 0 | 60 |
| `0B/29` | 132 | 81 / 81 / 81 | 0 / 0 | 59 |
| `18/00` | 931 | 929 / 929 / 929 | 0 / 0 | 59 |
| `1A/00` | 3 | 0 / 0 / 0 | 0 / 0 | 59 |
| `1A/1A` | 3 | 0 / 0 / 0 | 0 / 0 | 59 |
| `1A/1C` | 3 | 0 / 0 / 0 | 0 / 0 | 59 |
| `1A/04` | 3 | 0 / 0 / 0 | 0 / 0 | 58 |
| `1A/08` | 3 | 0 / 0 / 0 | 0 / 0 | 58 |
| `1A/0A` | 3 | 0 / 0 / 0 | 0 / 0 | 58 |
| `1A/18` | 3 | 0 / 0 / 0 | 0 / 0 | 58 |
| `1A/0E` | 3 | 0 / 0 / 0 | 0 / 0 | 57 |
| `1A/10` | 3 | 0 / 0 / 0 | 0 / 0 | 56 |
| `1A/06` | 3 | 0 / 0 / 0 | 0 / 0 | 55 |
| `07/03` | 67 | 14 / 14 / 14 | 0 / 0 | 53 |
| `1A/0C` | 3 | 1 / 1 / 1 | 0 / 0 | 53 |
| `11/07` | 701 | 684 / 684 / 684 | 0 / 0 | 51 |
| `13/05` | 51 | 0 / 51 / 51 | 51 / 51 | 51 |
| `1A/1E` | 3 | 1 / 1 / 1 | 0 / 0 | 51 |
| `0B/1C` | 132 | 92 / 92 / 92 | 0 / 0 | 48 |
| `1A/14` | 1 | 0 / 0 / 0 | 0 / 0 | 47 |
| `1A/02` | 1 | 0 / 0 / 0 | 0 / 0 | 46 |
| `1A/16` | 1 | 0 / 0 / 0 | 0 / 0 | 45 |
| `07/18` | 294 | 280 / 280 / 280 | 0 / 0 | 41 |
| `1A/20` | 1 | 0 / 0 / 0 | 0 / 0 | 41 |
| `09/12` | 16 | 0 / 0 / 0 | 0 / 0 | 32 |
| `12/08` | 4 | 0 / 0 / 0 | 0 / 0 | 32 |
| `0B/25` | 4 | 0 / 0 / 0 | 0 / 0 | 30 |
| `0B/00` | 12,309 | 12,296 / 12,296 / 12,296 | 0 / 0 | 29 |
| `0B/18` | 4 | 0 / 0 / 0 | 0 / 0 | 24 |
| `12/02` | 736 | 713 / 736 / 736 | 23 / 23 | 23 |
| `12/05` | 2,208 | 2,189 / 2,189 / 2,189 | 0 / 0 | 19 |
| `00/07` | 175 | 161 / 161 / 161 | 0 / 0 | 14 |
| `15/05` | 1 | 0 / 0 / 0 | 0 / 0 | 14 |
| `01/02` | 7 | 0 / 0 / 0 | 0 / 0 | 7 |
| `00/0A` | 3 | 0 / 0 / 0 | 0 / 0 | 6 |
| `12/09` | 720 | 716 / 716 / 716 | 0 / 0 | 4 |
| `13/04` | 3 | 0 / 3 / 3 | 3 / 3 | 3 |
| `00/02` | 1 | 0 / 0 / 0 | 0 / 0 | 2 |
| `00/06` | 84 | 82 / 82 / 82 | 0 / 0 | 2 |
| `04/01` | 2 | 0 / 0 / 0 | 0 / 0 | 2 |
| `00/00` | 1 | 0 / 0 / 0 | 0 / 0 | 1 |
| `03/01` | 1 | 0 / 0 / 0 | 0 / 0 | 1 |
| `07/0B` | 205 | 204 / 204 / 204 | 0 / 0 | 1 |
| `09/06` | 7,556 | 7,555 / 7,555 / 7,555 | 0 / 0 | 1 |
| `0E/01` | 69,382 | 69,381 / 69,381 / 69,381 | 0 / 0 | 1 |
| `19/01` | 274 | 273 / 273 / 273 | 0 / 0 | 1 |

Both complete runs and both summary gates exit 0 / **PASS**, with coverage of all 1,473,123 iterations and **zero unexplained WRAM, ordinary RNG, ordinary gameplay APU-read or unmatched-command differences**. Cartridge answers have 737 segments; live SPC has 742. The immutable-input/periodic-seed contract and representation exclusions remain those of the existing census. These are logical-state gates, not GPU/video or unseeded quest proof.

The cartridge-answer compared-WRAM partition is 1,472,899 clean + 224 lag-classified, with no remaining polygon WRAM misses in that run. Live SPC has 14,715 polygon-classified and 628 observed song-poll-classified WRAM misses, plus the same 224 lag cases; ordinary misses are zero. Existing accepted polygon/song classifications remain, including live polygon RNG alignment (19 unused / 19 extra) and song RNG alignment (193 / 192). These counts are reported as classified differences, not equality. The **224 lag cases remain** (the existing 220 rupee-queue cases and four map/flute sampling overruns); their removal requires the unfinished text pacing work. Tightening the lag-start rule did not erase them.

Raw logs, terminal gates and category counters are `census-{rom,live}.log`, `census-{rom,live}-summary.txt`, and `pacing-metrics.json` under `target/native-pacing/`. The earlier 348k prefix is retained separately and is not a full-pass receipt.

### Remaining work (loading before text)

- Overworld properties/overlay/build (`08/00..02`): **9,748** total (**6,466 + 706 + 2,576**), including the song-bank caller. The follow-up below adds cartridge-sized, NMI-disabled song-upload elapsed time. Decompression, conversion, map/tile build budgets and retained publication remain unfinished; this category still has no exact-count loading matches.
- Mirror warp (`09/23`): **6,054**. Its graphics, overlay and bank work needs the same reusable work accounting.
- Overworld map/GFX (`09/01`, `09/03`, `09/04`): **8,359** total (**3,479 + 4,372 + 508**). Count actual decompression and map/tile stores, with retained publication positions.
- VWF text and message scroll (`0E/02`): **12,964 lag NMIs**, 4,418 lagged iterations. Retain glyph setup/drawing and pixel-copy progress, derive a budget from the actual column/plane work and scroll words, and place NMI sound delivery between the correct operations. The rupee cadence repair remains in this category.
- Dungeon room transitions (`07/02`): **8,770**, and initial dungeon loading (`06/00`): **8,196**. Preserve decompression, tile/object draw, graphics expansion, and ready-upload boundaries; holding only after a complete load is insufficient for faithful partial state.
- Spiral stairs (`07/0E`): **7,979**. Native room/graphics work and its caller need retained slices.
- Falling/warp/stair and map/menu work: remaining context sizes are in the complete inventory above. Polygon-active lag is the overlapping **3,939**-NMI view; existing internal worker progress is not proof of real elapsed callback pacing.
- Iris: retain partial table/copy/caller progress and reduce the coarse budget's remaining count errors. This patch restores measured crystal iris duration but does not establish table-by-table scanout parity.
- Gameplay load: the **4,961** lag NMIs above remain outside this first deterministic-work batch.

No runtime ledger lag counts, per-scene duration tables, new exact cycle model, GPU run, `--video`, cached-A/V run, Snes9x-core run, `cargo fmt`, commit or push was used. Scratch analysis scripts are under `/tmp`; no one-off script was added to the repository. The supplied ledger/cache and the other checkout were read-only.


### Reproduce the full CPU gates

```sh
LEDGER=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
BIN=target/native-pacing/final-reduced-zelda3
ZELDA3_SAVE_DIR=/tmp/native-pacing-repro-rom "$BIN" --logical-replay "$LEDGER" "$C/initial.srm" \
  --report 0 --seed-every 2000 > /tmp/native-pacing-repro-rom.log
python3 scripts/logical_census_summary.py /tmp/native-pacing-repro-rom.log --gate --expect-iterations 1473123
ZELDA3_SAVE_DIR=/tmp/native-pacing-repro-live "$BIN" --logical-replay "$LEDGER" "$C/initial.srm" \
  --report 0 --seed-every 2000 --live-apu-reads > /tmp/native-pacing-repro-live.log
python3 scripts/logical_census_summary.py /tmp/native-pacing-repro-live.log --gate --expect-iterations 1473123
```

All evidence named without a path prefix above is under `target/native-pacing/`. Retained binary/baseline hashes are in `artifact-hashes.json`; `engine-change.patch` and `harness-change.patch` retain the source changes. These are local CPU evidence, with no promotion, commit or deployment claim. The lag-free state ledger is intentionally changed and was not constrained to its old hash stream.


### Final validation

| Check | Result |
| --- | --- |
| Default and reduced ordinary binary builds | PASS, exit 0, no warnings |
| Default mode versus freshly recorded main baseline | PASS, 200,000 records and EOF |
| Default Zelda library suite | 2,456 passed, 1 permitted existing spotlight failure, 3 ignored; exit 101 |
| Reduced Zelda library suite | 1,991 passed, 0 failed, 2 ignored; all 1,993 test names accounted for |
| SNES library suites, both feature configurations | 504 passed, 0 failed, 7 ignored each; exit 0 |
| Full cartridge-answer census / gate | Exit 0 / PASS, all 1,473,123 iterations, unexplained 0 |
| Full live-SPC census / gate | Exit 0 / PASS, all 1,473,123 iterations, unexplained 0 |
| Eight uninterrupted crystal/pendant outcomes | PASS, stable room/position and complete save-block match |
| Iris phase duration target | PASS within 1 frame in all eight probes; exact for all seven crystals |
| Complete native scene pacing / partial-progress fidelity | NOT MET; remaining work and sizes listed above |
| `git diff --check` | PASS |

The sole default failure is `cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`, the permitted existing failure. Default lib-test compilation retains the same 24 existing unused-fixture warnings documented in the earlier reports; reduced lib-test compilation and both ordinary binary builds have no warnings. No new fixture warning was introduced.

The reduced suite's first single-thread run passed 87 tests before being interrupted to resume the remaining tests with four CPU threads. That remainder exited 0 with 1,904 passed and two ignored. `reduced-lib-coverage.json` checks the union against the executable's complete 1,993-name test list: none missing and none extra. The prefix is not represented as a standalone passing suite. Default runs as one complete four-thread invocation.

Logs and hashes remain in `target/native-pacing/`; `verification.json` records final counts and scope. HEAD remains `b5dde888` on `codex/native-pacing`, with changes left uncommitted.


## Follow-up at main `56a2fadc`: song-upload elapsed time

**This batch is incomplete. Neither requested category is finished.** The retained change covers the song-bank elapsed substep only. The loading work-budget trial failed the pendant control and was removed; category 2 (VWF and message scroll) was not started. The 1–2-frame loading target remains unmet.

### Retained implementation and rate source

Lag-free runtime overworld uploads now use the existing `kSpcRuntimeUpload0` cartridge stream instead of the complete semantic sample-bank image. The sidecar is **13,378 bytes**, with **13,366 payload bytes** in two blocks (11,694 to `$D000`, 1,672 to `$2B00`), versus the semantic image's 50,066 bytes. `song-stream.json` verifies every sidecar byte against the ROM at `$1A:9EF5`. The first trial used the complete image and took 85–86 elapsed callbacks; that result was rejected.

Elapsed duration follows the existing retained `SongBankHostTransfer` sender and actual SPC acknowledgements. The rate source is `$80:8888 LoadSongBank`, documented by the `SONG_BANK_*` constants and `perform_host_access` in `spc_driver_clock.rs`: an ordinary byte has 6 master clocks from counter to value, 236 to its first acknowledgement poll, and 82 from a successful poll to the next counter (**324 clocks plus 52 for each failed poll**). The receiver's real progress determines how many polls occur. Headers, the final byte and port clears retain their own existing protocol phases. No fixed byte-to-frame multiplier, new CPU cycle model or scene-duration table was added.

A masked callback advances audio and the presentation/debug host only. It skips main, input recording/consumption, gameplay RNG advancement and NMI. Queued NMI sound remains pending. When the byte receiver finishes, the next callback delivers the closing NMI once. Foreign seeding/reset and changing lag-free mode clear the pacing state. The new unit test checks command retention, recorder/main preservation and single closing-NMI delivery. The byte cursor and receiver remain owned by the existing audio clock; this adds no serialized `ZeldaState` field. The callback marker is serde-skipped like iris holds; mid-upload checkpoint resume is not verified.

`native_presentation_pending` lets the CPU harness drain both kinds of callback. `native_held` counts accepted NMIs beyond the ordinary closing NMI; **`native_elapsed` counts masked callbacks separately**. Segment/category output and the summary gate expose both counters. Thus song-upload elapsed time cannot increase the reported restoration of accepted lag NMIs. The first loading phase in each of the eight principal probes has 25–26 masked callbacks. Mid-host return placement and full load/caller suspension are still not established.

### Fresh baseline and full CPU gates

The default binary was built and copied from clean main `56a2fadc` before edits. It recorded `main-default-200k.jsonl.zst` from the same cache inputs and SRAM. The changed default binary matches all **200,000 state/display/SRAM/PCM/input/context records and EOF**. Both ordinary builds are warning-free.

The main cartridge-answer census was rerun completely. Every pacing category matches the preceding batch's archived result. For the before live-SPC counters, the archived full receipt was reused after verifying its hashes and reconstructing its source patches: all six runtime/harness files reproduce `56a2fadc` exactly (`before-source-verification.json`). The optional fresh before-live prefix was interrupted and is not claimed as a full receipt. Both **changed-binary** complete runs exited 0, and both terminal summary gates are **PASS**, with all 1,473,123 iterations and zero unexplained state/RNG/gameplay-read/unmatched-command differences. These remain periodically seeded logical-state gates, not video or unseeded quest proof.

| Answer mode | Exact-count iterations before → after | Held NMIs before → after | Source lag NMIs | New masked elapsed frames |
| --- | ---: | ---: | ---: | ---: |
| Cartridge | 1,443,663 → 1,443,663 / 1,473,123 | 11,178 → 11,178 | 102,677 | 2,843 |
| Live SPC | 1,443,658 → 1,443,658 / 1,473,123 | 11,176 → 11,176 | 102,677 | 2,815 |

The accepted-NMI equality and hold totals are unchanged. This change restores a separate elapsed interval; it does **not** restore the requested loading or text lag NMIs.

Below, paired values are cartridge / live-SPC. Exact-count and held values are identical before and after; elapsed values are after (zero before).

| Entry context | Iterations | Exact-count matches before = after | Held NMIs before = after | Source lag NMIs | Masked elapsed frames |
| --- | ---: | ---: | ---: | ---: | ---: |
| `08/00` | 157 | 0 / 0 | 0 / 1 | 6,466 | 2,719 / 2,616 |
| `08/01` | 157 | 0 / 0 | 0 / 1 | 706 | 0 / 25 |
| `08/02` | 157 | 0 / 0 | 0 / 1 | 2,576 | 0 / 0 |
| `09/01` | 251 | 0 / 0 | 0 / 0 | 3,479 | 0 / 0 |
| `09/03` | 251 | 0 / 0 | 0 / 0 | 4,372 | 0 / 0 |
| `09/04` | 251 | 0 / 0 | 0 / 0 | 508 | 0 / 0 |
| `09/23` | 12,279 | 11,100 / 11,100 | 0 / 0 | 6,054 | 0 / 0 |
| `0E/02` | 58,097 | 53,679 / 53,679 | 0 / 0 | 12,964 | 0 / 0 |

Of all masked frames, 2,719 / 2,641 occur in `08/00..02`; the remaining 124 / 174 occur in other callers of the same bank routine. All category rows are retained in `pacing-metrics.json`. The three existing live-SPC holds in `08/00..02` are unchanged boundary iris work, not a loading repair.

Both runs retain **220 `$0CFD` exceptions plus four existing map/flute sampling cases**. Cartridge WRAM has 1,472,899 clean iterations plus those 224 classified cases. Live SPC additionally has 14,714 polygon-classified and 627 song-poll-classified differences (before: 14,715 and 628); these are classifications, not equality. Existing live polygon RNG alignment is 19 unused / 19 extra; song alignment is 193 / 192. None of the exception rules was broadened.

### Crystal phase table

Each cell is **original host frames / fresh main-56 native / changed native**. Changed durations include masked elapsed frames. All 48 before-phase durations were checked against a fresh main-binary probe.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 1 / 26 | 5 / 1 / 1 | 17 / 1 / 1 | 2 / 2 / 2 | 35 / 34 / 34 | 16 / 16 / 16 |
| 380000 | 84 / 1 / 26 | 7 / 1 / 1 | 24 / 1 / 1 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 460000 | 84 / 1 / 27 | 7 / 1 / 1 | 24 / 1 / 1 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 626000 | 91 / 1 / 26 | 9 / 1 / 1 | 24 / 1 / 1 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 734000 | 84 / 1 / 26 | 7 / 1 / 1 | 25 / 1 / 1 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 880000 | 84 / 1 / 26 | 7 / 1 / 1 | 26 / 1 / 1 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 1064000 | 84 / 1 / 26 | 7 / 1 / 1 | 25 / 1 / 1 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 1316000 | 84 / 1 / 26 | 7 / 1 / 1 | 25 / 1 / 1 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |

All eight stable native boundaries remain unchanged: 95163, 383667, 465848, 629876, 738219, 884358, 1066994 and 1321195. Each still matches the source's stable room, Link position and complete `$F000..$F4FF` save block within the existing 2,000-iteration window. The final crystal's source stable match remains 1320931. These are shifted logical outcomes, not new display acceptance.

The removed load-budget trial counted decoded bytes, graphics plane stores, map stores/definition blocks and sprite scan cells. Its first pendant properties phase took 82 frames against 65 in the source; build took 26 against 17. Most crystal phases were much closer, so a universal post-call hold multiplier did not establish the required timing. Adding compressed-input counts did not resolve the control. The trial's `load-probe*` / `load-input*` artifacts are diagnostic only; **no trial loading budget remains in production code**. Retained operation/publication boundaries and the work between them still need to be implemented and verified.

### Rupee cadence and remaining order

The fresh live-SPC `162000–163000` probe retains PCM and command requests. The first text run still requests rupee SFX `$29` on port 2 every **8 frames** (iterations 162443, 162451, …, 162651). For 24 measured PCM onset gaps, the existing 7.9–8.7 kHz band/envelope method gives approximately **7.165–8.375 frames**, median 8.122. This remains the compressed cadence; the source drawing intervals of 23–27 frames have not been restored. `rupee-gaps.json` records the requests, samples and gaps. This is an onset diagnostic, not exact PCM acceptance.

1. Finish loading first: `08/00..02` (9,748 lag NMIs), shared `09/01,03,04` (8,359), and mirror `09/23` (6,054). Retain decompression/conversion, map/tile stores and ready-upload progress; derive work budgets at those operations, and meet the pendant/crystal phase controls before continuing.
2. Then implement retained VWF glyph column/plane and message-scroll word progress in `0E/02` (12,964), with NMI sound delivery between operations. Recheck rupee requests/PCM and remove `$0CFD` exceptions only when the byte actually reconverges.

### Verification at this stopping point

| Check | Result |
| --- | --- |
| Default and reduced ordinary builds | PASS; no warnings |
| Fresh main default baseline, 200k records + EOF | PASS; every record unchanged |
| Default Zelda lib | 2,457 passed; only the known spotlight failure; 3 ignored |
| Reduced Zelda lib | 1,992 passed; 0 failed; 2 ignored; all 1,994 names accounted for |
| SNES libs, both configurations | 504 passed, 0 failed, 7 ignored each |
| Both complete changed-binary censuses / gates | Exit 0 / PASS; 1,473,123 iterations each |
| Masked upload command/closing-NMI regression | PASS |
| Eight stable crystal/pendant outcomes | PASS |
| Loading duration target | NOT MET |
| VWF / rupee cadence fix | NOT IMPLEMENTED |
| `git diff --check` | PASS |

Default lib-test compilation retains exactly the same 24 existing fixture warnings; reduced lib-test compilation is warning-free. The reduced coverage receipt explains its original long-running fixtures and separately run remainder; an interrupted process is not represented as a standalone passing suite.

All new evidence is under `target/native-pacing2/`: binary/build logs, fresh baseline, complete censuses and terminal summaries, classifications, category metrics, crystal and rupee traces, coverage receipts and artifact hashes. Reproduce the full gates with the commands above using `BIN=target/native-pacing2/final-reduced-zelda3` and fresh separate save directories. The ledger/cache and other checkout remained read-only. No GPU or libretro/Snes9x-core run, `--video`, cached-A/V run, cargo GPU test, `cargo fmt`, repository scratch script, commit or push was used. HEAD remains `56a2fadc` on `codex/native-pacing2`.
