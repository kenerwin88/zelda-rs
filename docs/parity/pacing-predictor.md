# Cycle-ledger pacing predictor

**The existing cost counter is not a viable general pacing rule by itself.** It provides useful work estimates for some charged loading/text paths, but neither the primary rule nor the two generic refinements recovers the route’s lag counts reliably. No lagless behavior was changed.

The complete cartridge-answer census covered **1,473,123 iterations**, including **39,575 lagged iterations** and **102,677 source lag NMIs**. The logical-state census gate **PASSes**. Both default and reduced ordinary builds finished warning-free.

## Overall count accuracy

Exact and within-one rates below use only the 39,575 **lagged iterations**. MAE is mean absolute count error on those iterations; the all-iteration MAE is shown separately. Predicted/native totals include all iterations, including false positives.

| Rule | Exact lagged iterations | Within one lagged iterations | Predicted / actual lag frames | MAE lagged / all | False-positive zero-lag iterations |
| --- | ---: | ---: | ---: | ---: | ---: |
| P0: main cost + entry handler | 16,581/39,575 (41.90%) | 36,540/39,575 (92.33%) | 65,092 / 102,677 | 0.950 / 0.02551 | 0 |
| P1: repeat handler overhead | 19,057/39,575 (48.15%) | 37,049/39,575 (93.62%) | 70,082 / 102,677 | 0.850 / 0.02283 | 0 |
| P2: P1 + refresh | 21,385/39,575 (54.04%) | 37,094/39,575 (93.73%) | 73,700 / 102,677 | 0.780 / 0.02096 | 0 |
| Production iris + song uploader (accepted holds) | 10,115/39,575 (25.56%) | 30,283/39,575 (76.52%) | 11,178 / 102,677 | 2.312 / 0.06211 | 0 |

A lag frame cannot independently be “within one”: the tolerance is on its iteration’s count. The following weights every qualifying iteration by its **actual source lag count**, so the denominator is exactly 102,677. These are source frames belonging to exactly/approximately predicted iterations, not a claim of matched NMI placement.

| Rule | Source lag frames in exact-count iterations | Source lag frames in within-one iterations | Sum of min(prediction, actual) | Under / over frames |
| --- | ---: | ---: | ---: | ---: |
| P0: main cost + entry handler | 23,587 (22.97%) | 56,661 (55.18%) | 65,092 | 37,585 / 0 |
| P1: repeat handler overhead | 33,082 (32.22%) | 66,938 (65.19%) | 69,565 | 33,112 / 517 |
| P2: P1 + refresh | 39,514 (38.48%) | 66,249 (64.52%) | 72,747 | 29,930 / 953 |
| Production iris + song uploader (accepted holds) | 10,115 (9.85%) | 31,342 (30.52%) | 11,178 | 91,499 / 0 |

Production still inserts **11,178 accepted holds** and **2,843 masked song-upload callbacks**. The latter are excluded from all accepted-lag totals. The accepted-hold and masked-elapsed totals match the prior cartridge census in `native-pacing.md`; the predictor does not drive either counter.

P0’s all-iteration exact rate is 98.44%, which is dominated by the 1,433,548 zero-lag calls. It is not the success rate on lag. Even counts that agree do not establish partial-state publication or audio/event placement inside an iteration.

## Category inventory and production comparison

Same opening module/submodule, owner, work kind and lag-descending order as the inventory in `native-pacing.md`. Every inventory iteration/lagged/lag-total cell was checked against the new CSV. P0 and production MAEs are on lagged iterations; totals include zero-lag iterations. Categories without source lag remain in overall metrics.

| Entry | Source routine / owner | Work kind | Iterations | Lagged | Actual lag NMIs | Share | Cumulative | P0 frames | P0 exact lagged | P0 MAE | Native held | Native exact lagged | Native MAE | Masked elapsed |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | RenderText (VWF, message scroll) | Scene: text / scroll | 58,097 | 4,418 | 12,964 | 12.63% | 12.63% | 10,245 | 1,741/4,418 (39.41%) | 0.615 | 0 | 0/4,418 (0.00%) | 2.934 | 0 |
| `07/02` | module07_02_supertile_transition | Scene: transition / UI / cutscene | 33,311 | 1,976 | 8,770 | 8.54% | 21.17% | 6,600 | 776/1,976 (39.27%) | 1.098 | 0 | 0/1,976 (0.00%) | 4.438 | 0 |
| `06/00` | module_pre_dungeon | Scene: load / upload | 142 | 142 | 8,196 | 7.98% | 29.15% | 7,272 | 0/142 (0.00%) | 6.507 | 0 | 0/142 (0.00%) | 57.718 | 0 |
| `07/0E` | module07_0e_spiral_stairs | Scene: transition / UI / cutscene | 27,788 | 4,952 | 7,979 | 7.77% | 36.92% | 7,391 | 4,663/4,952 (94.16%) | 0.119 | 0 | 0/4,952 (0.00%) | 1.611 | 0 |
| `08/00` | pre_overworld_load_properties (incl. song bank) | Scene: load / upload | 157 | 157 | 6,466 | 6.30% | 43.22% | 5,551 | 0/157 (0.00%) | 5.828 | 0 | 0/157 (0.00%) | 41.185 | 2,719 |
| `09/23` | module09_mirror_warp | Scene: load / upload | 12,279 | 1,179 | 6,054 | 5.90% | 49.11% | 3,522 | 366/1,179 (31.04%) | 2.148 | 0 | 0/1,179 (0.00%) | 5.135 | 0 |
| `0F/01` | module0f_spotlight_close | Scene: iris | 4,879 | 4,879 | 5,432 | 5.29% | 54.40% | 1,368 | 957/4,879 (19.61%) | 0.833 | 4,879 | 4,326/4,879 (88.67%) | 0.113 | 0 |
| `09/03` | module09_load_new_map_and_gfx | Scene: load / upload | 251 | 251 | 4,372 | 4.26% | 58.66% | 745 | 0/251 (0.00%) | 14.450 | 0 | 0/251 (0.00%) | 17.418 | 0 |
| `07/07` | module07_07_falling_transition | Scene: transition / UI / cutscene | 5,591 | 2,919 | 4,060 | 3.95% | 62.62% | 2,550 | 1,523/2,919 (52.18%) | 0.517 | 0 | 0/2,919 (0.00%) | 1.391 | 0 |
| `07/00` | module07_00_player_control | Gameplay load | 780,760 | 2,960 | 3,679 | 3.58% | 66.20% | 1,738 | 1,019/2,960 (34.43%) | 0.656 | 0 | 0/2,960 (0.00%) | 1.243 | 0 |
| `09/01` | module09_load_aux_gfx | Scene: load / upload | 251 | 251 | 3,479 | 3.39% | 69.59% | 3,269 | 42/251 (16.73%) | 0.837 | 0 | 0/251 (0.00%) | 13.861 | 0 |
| `07/0F` | module07_0f_landing_wipe | Scene: iris | 2,718 | 2,718 | 2,992 | 2.91% | 72.50% | 2,021 | 1,747/2,718 (64.28%) | 0.357 | 2,718 | 2,454/2,718 (90.29%) | 0.101 | 0 |
| `10/01` | module10_spotlight_open | Scene: iris | 2,669 | 2,669 | 2,672 | 2.60% | 75.10% | 231 | 228/2,669 (8.54%) | 0.915 | 2,669 | 2,666/2,669 (99.89%) | 0.001 | 0 |
| `0E/03` | module0e_03_dungeon_map | Scene: transition / UI / cutscene | 13,047 | 115 | 2,672 | 2.60% | 77.71% | 2,475 | 0/115 (0.00%) | 1.713 | 0 | 0/115 (0.00%) | 23.235 | 0 |
| `08/02` | overworld_load_and_build_screen | Scene: load / upload | 157 | 157 | 2,576 | 2.51% | 80.22% | 0 | 0/157 (0.00%) | 16.408 | 0 | 0/157 (0.00%) | 16.408 | 0 |
| `07/15` | module07_15_warp_pad | Scene: transition / UI / cutscene | 2,900 | 1,735 | 2,333 | 2.27% | 82.49% | 1,983 | 1,437/1,735 (82.82%) | 0.202 | 0 | 0/1,735 (0.00%) | 1.345 | 0 |
| `19/02` | module19_triforce_room | Scene: polygon / cutscene | 3,559 | 1,855 | 2,001 | 1.95% | 84.44% | 15 | 0/1,855 (0.00%) | 1.071 | 0 | 0/1,855 (0.00%) | 1.079 | 0 |
| `0E/0A` | module0e_0a_flute_menu | Scene: transition / UI / cutscene | 10,081 | 90 | 1,845 | 1.80% | 86.23% | 943 | 0/90 (0.00%) | 10.022 | 0 | 0/90 (0.00%) | 20.500 | 0 |
| `09/00` | module09_00_player_control | Gameplay load | 303,167 | 1,192 | 1,253 | 1.22% | 87.45% | 75 | 14/1,192 (1.17%) | 0.988 | 0 | 0/1,192 (0.00%) | 1.051 | 0 |
| `0E/07` | messaging_overworld_map | Scene: transition / UI / cutscene | 4,441 | 56 | 1,065 | 1.04% | 88.49% | 866 | 0/56 (0.00%) | 3.554 | 0 | 0/56 (0.00%) | 19.018 | 0 |
| `07/06` | module07_06_fat_inter_room_stairs | Scene: transition / UI / cutscene | 1,360 | 603 | 984 | 0.96% | 89.45% | 884 | 549/603 (91.04%) | 0.166 | 0 | 0/603 (0.00%) | 1.632 | 0 |
| `11/00` | module11_dungeon_falling_entrance | Scene: load / upload | 1,079 | 17 | 973 | 0.95% | 90.40% | 881 | 0/17 (0.00%) | 5.412 | 0 | 0/17 (0.00%) | 57.235 | 0 |
| `07/01` | module07_01_subtile_transition | Scene: transition / UI / cutscene | 32,647 | 928 | 928 | 0.90% | 91.30% | 916 | 916/928 (98.71%) | 0.013 | 0 | 0/928 (0.00%) | 1.000 | 0 |
| `08/01` | pre_overworld_load_overlays | Scene: load / upload | 157 | 157 | 706 | 0.69% | 91.99% | 0 | 0/157 (0.00%) | 4.497 | 0 | 0/157 (0.00%) | 4.497 | 0 |
| `05/00` | module05_load_file | Scene: transition / UI / cutscene | 11 | 11 | 639 | 0.62% | 92.61% | 479 | 0/11 (0.00%) | 14.545 | 0 | 0/11 (0.00%) | 58.091 | 0 |
| `09/04` | module09_load_new_sprites | Scene: load / upload | 251 | 251 | 508 | 0.49% | 93.11% | 0 | 0/251 (0.00%) | 2.024 | 0 | 0/251 (0.00%) | 2.024 | 0 |
| `12/03` | module12_game_over | Scene: transition / UI / cutscene | 391 | 391 | 472 | 0.46% | 93.57% | 10 | 0/391 (0.00%) | 1.182 | 391 | 310/391 (79.28%) | 0.207 | 0 |
| `0F/00` | module0f_spotlight_close | Scene: iris | 287 | 287 | 449 | 0.44% | 94.00% | 287 | 125/287 (43.55%) | 0.564 | 287 | 125/287 (43.55%) | 0.564 | 0 |
| `07/12` | module07_11_straight_interroom_stairs | Scene: transition / UI / cutscene | 1,026 | 239 | 391 | 0.38% | 94.38% | 298 | 162/239 (67.78%) | 0.389 | 0 | 0/239 (0.00%) | 1.636 | 0 |
| `09/2E` | module09_2e_whirlpool | Scene: load / upload | 895 | 100 | 363 | 0.35% | 94.74% | 142 | 2/100 (2.00%) | 2.210 | 0 | 0/100 (0.00%) | 3.630 | 0 |
| `07/16` | module07_16_update_pegs | Scene: transition / UI / cutscene | 1,616 | 101 | 303 | 0.30% | 95.03% | 0 | 0/101 (0.00%) | 3.000 | 0 | 0/101 (0.00%) | 3.000 | 0 |
| `09/11` | module09_load_new_map_and_gfx | Scene: load / upload | 16 | 16 | 274 | 0.27% | 95.30% | 47 | 0/16 (0.00%) | 14.188 | 0 | 0/16 (0.00%) | 17.125 | 0 |
| `09/16` | module09_fade_back_in_from_mosaic | Scene: transition / UI / cutscene | 528 | 233 | 265 | 0.26% | 95.56% | 55 | 23/233 (9.87%) | 0.901 | 0 | 0/233 (0.00%) | 1.137 | 0 |
| `09/0D` | overworld_start_mosaic_transition | Scene: transition / UI / cutscene | 528 | 223 | 247 | 0.24% | 95.80% | 40 | 16/223 (7.17%) | 0.928 | 0 | 0/223 (0.00%) | 1.108 | 0 |
| `0E/0B` | save_menu | Scene: transition / UI / cutscene | 1,351 | 81 | 243 | 0.24% | 96.03% | 217 | 55/81 (67.90%) | 0.321 | 0 | 0/81 (0.00%) | 3.000 | 0 |
| `09/0F` | module09_load_aux_gfx | Scene: load / upload | 16 | 16 | 242 | 0.24% | 96.27% | 224 | 0/16 (0.00%) | 1.125 | 0 | 0/16 (0.00%) | 15.125 | 0 |
| `01/00` | module01_file_select | Scene: transition / UI / cutscene | 4 | 4 | 235 | 0.23% | 96.50% | 141 | 0/4 (0.00%) | 23.500 | 0 | 0/4 (0.00%) | 58.750 | 0 |
| `09/2C` | module09_mirror_warp | Scene: load / upload | 477 | 36 | 228 | 0.22% | 96.72% | 141 | 16/36 (44.44%) | 2.417 | 0 | 0/36 (0.00%) | 6.333 | 0 |
| `10/00` | module10_spotlight_open | Scene: iris | 157 | 157 | 157 | 0.15% | 96.87% | 0 | 0/157 (0.00%) | 1.000 | 157 | 157/157 (100.00%) | 0.000 | 0 |
| `17/01` | module17_save_and_quit | Scene: transition / UI / cutscene | 45 | 3 | 141 | 0.14% | 97.01% | 102 | 0/3 (0.00%) | 13.000 | 0 | 0/3 (0.00%) | 47.000 | 74 |
| `1B/02` | module1b_spawn_select | Scene: transition / UI / cutscene | 263 | 9 | 139 | 0.14% | 97.15% | 123 | 5/9 (55.56%) | 1.778 | 0 | 0/9 (0.00%) | 15.444 | 0 |
| `09/20` | overworld_load_overlays2 | Scene: load / upload | 28 | 28 | 138 | 0.13% | 97.28% | 0 | 0/28 (0.00%) | 4.929 | 0 | 0/28 (0.00%) | 4.929 | 0 |
| `0E/05` | module0e_05_desert_prayer | Scene: transition / UI / cutscene | 122 | 119 | 138 | 0.13% | 97.42% | 2 | 1/119 (0.84%) | 1.143 | 0 | 0/119 (0.00%) | 1.160 | 0 |
| `00/01` | module00_intro | Scene: polygon / cutscene | 25 | 11 | 114 | 0.11% | 97.53% | 46 | 1/11 (9.09%) | 6.182 | 0 | 0/11 (0.00%) | 10.364 | 0 |
| `0B/24` | overworld_start_mosaic_transition | Scene: transition / UI / cutscene | 132 | 93 | 103 | 0.10% | 97.63% | 12 | 2/93 (2.15%) | 0.978 | 0 | 0/93 (0.00%) | 1.108 | 0 |
| `09/21` | overworld_load_ambient_overlay_false | Scene: transition / UI / cutscene | 28 | 28 | 84 | 0.08% | 97.71% | 0 | 0/28 (0.00%) | 3.000 | 0 | 0/28 (0.00%) | 3.000 | 0 |
| `09/0E` | pre_overworld_load_overlays | Scene: load / upload | 16 | 16 | 83 | 0.08% | 97.79% | 0 | 0/16 (0.00%) | 5.188 | 0 | 0/16 (0.00%) | 5.188 | 0 |
| `15/07` | module15_mirror_warp_from_aga | Scene: transition / UI / cutscene | 2,843 | 79 | 83 | 0.08% | 97.87% | 83 | 79/79 (100.00%) | 0.000 | 0 | 0/79 (0.00%) | 1.051 | 0 |
| `19/00` | module19_triforce_room | Scene: polygon / cutscene | 358 | 4 | 72 | 0.07% | 97.94% | 37 | 0/4 (0.00%) | 8.750 | 0 | 0/4 (0.00%) | 18.000 | 0 |
| `07/13` | module07_11_straight_interroom_stairs | Scene: transition / UI / cutscene | 166 | 47 | 71 | 0.07% | 98.01% | 69 | 46/47 (97.87%) | 0.043 | 0 | 0/47 (0.00%) | 1.511 | 0 |
| `0B/17` | overworld_start_mosaic_transition | Scene: transition / UI / cutscene | 132 | 65 | 68 | 0.07% | 98.08% | 5 | 2/65 (3.08%) | 0.969 | 0 | 0/65 (0.00%) | 1.046 | 0 |
| `0B/28` | overworld_load_and_build_screen | Scene: load / upload | 4 | 4 | 68 | 0.07% | 98.14% | 0 | 0/4 (0.00%) | 17.000 | 0 | 0/4 (0.00%) | 17.000 | 0 |
| `0B/2B` | overworld_func2b | Scene: transition / UI / cutscene | 67 | 66 | 66 | 0.06% | 98.21% | 6 | 6/66 (9.09%) | 0.909 | 0 | 0/66 (0.00%) | 1.000 | 0 |
| `0B/19` | overworld_func19 | Scene: transition / UI / cutscene | 4 | 4 | 64 | 0.06% | 98.27% | 0 | 0/4 (0.00%) | 16.000 | 0 | 0/4 (0.00%) | 16.000 | 0 |
| `15/03` | module15_mirror_warp_from_aga | Scene: transition / UI / cutscene | 114 | 12 | 64 | 0.06% | 98.33% | 36 | 4/12 (33.33%) | 2.333 | 0 | 0/12 (0.00%) | 5.333 | 0 |
| `00/05` | module00_intro | Scene: polygon / cutscene | 123 | 62 | 62 | 0.06% | 98.39% | 0 | 0/62 (0.00%) | 1.000 | 0 | 0/62 (0.00%) | 1.000 | 0 |
| `0B/26` | module09_load_aux_gfx | Scene: load / upload | 4 | 4 | 61 | 0.06% | 98.45% | 56 | 0/4 (0.00%) | 1.250 | 0 | 0/4 (0.00%) | 15.250 | 0 |
| `0B/1A` | module09_load_aux_gfx | Scene: transition / UI / cutscene | 4 | 4 | 60 | 0.06% | 98.51% | 56 | 0/4 (0.00%) | 1.000 | 0 | 0/4 (0.00%) | 15.000 | 0 |
| `1A/12` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 60 | 0.06% | 98.57% | 35 | 0/3 (0.00%) | 8.333 | 0 | 0/3 (0.00%) | 20.000 | 0 |
| `0B/29` | module09_fade_back_in_from_mosaic | Scene: transition / UI / cutscene | 132 | 51 | 59 | 0.06% | 98.62% | 13 | 5/51 (9.80%) | 0.902 | 0 | 0/51 (0.00%) | 1.157 | 0 |
| `18/00` | module18_ganon_emerges | Scene: transition / UI / cutscene | 931 | 2 | 59 | 0.06% | 98.68% | 35 | 0/2 (0.00%) | 12.000 | 0 | 0/2 (0.00%) | 29.500 | 26 |
| `1A/00` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 59 | 0.06% | 98.74% | 35 | 0/3 (0.00%) | 8.000 | 0 | 0/3 (0.00%) | 19.667 | 0 |
| `1A/1A` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 59 | 0.06% | 98.80% | 35 | 0/3 (0.00%) | 8.000 | 0 | 0/3 (0.00%) | 19.667 | 0 |
| `1A/1C` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 59 | 0.06% | 98.85% | 35 | 0/3 (0.00%) | 8.000 | 0 | 0/3 (0.00%) | 19.667 | 0 |
| `1A/04` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 58 | 0.06% | 98.91% | 36 | 0/3 (0.00%) | 7.333 | 0 | 0/3 (0.00%) | 19.333 | 0 |
| `1A/08` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 58 | 0.06% | 98.97% | 35 | 0/3 (0.00%) | 7.667 | 0 | 0/3 (0.00%) | 19.333 | 0 |
| `1A/0A` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 58 | 0.06% | 99.02% | 35 | 0/3 (0.00%) | 7.667 | 0 | 0/3 (0.00%) | 19.333 | 0 |
| `1A/18` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 58 | 0.06% | 99.08% | 36 | 0/3 (0.00%) | 7.333 | 0 | 0/3 (0.00%) | 19.333 | 0 |
| `1A/0E` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 57 | 0.06% | 99.14% | 35 | 0/3 (0.00%) | 7.333 | 0 | 0/3 (0.00%) | 19.000 | 0 |
| `1A/10` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 56 | 0.05% | 99.19% | 35 | 0/3 (0.00%) | 7.000 | 0 | 0/3 (0.00%) | 18.667 | 0 |
| `1A/06` | module1_a_credits | Scene: polygon / cutscene | 3 | 3 | 55 | 0.05% | 99.24% | 35 | 0/3 (0.00%) | 6.667 | 0 | 0/3 (0.00%) | 18.333 | 0 |
| `07/03` | module07_03_overlay_change | Scene: transition / UI / cutscene | 67 | 53 | 53 | 0.05% | 99.30% | 0 | 0/53 (0.00%) | 1.000 | 0 | 0/53 (0.00%) | 1.000 | 0 |
| `1A/0C` | module1_a_credits | Scene: polygon / cutscene | 3 | 2 | 53 | 0.05% | 99.35% | 35 | 0/2 (0.00%) | 9.000 | 0 | 0/2 (0.00%) | 26.500 | 0 |
| `11/07` | module11_dungeon_falling_entrance | Scene: load / upload | 701 | 17 | 51 | 0.05% | 99.40% | 46 | 12/17 (70.59%) | 0.294 | 0 | 0/17 (0.00%) | 3.000 | 0 |
| `13/05` | module13_boss_victory_pendant | Scene: transition / UI / cutscene | 51 | 51 | 51 | 0.05% | 99.45% | 0 | 0/51 (0.00%) | 1.000 | 51 | 51/51 (100.00%) | 0.000 | 0 |
| `1A/1E` | module1_a_credits | Scene: polygon / cutscene | 3 | 2 | 51 | 0.05% | 99.50% | 35 | 0/2 (0.00%) | 8.000 | 0 | 0/2 (0.00%) | 25.500 | 0 |
| `0B/1C` | overworld_func1c | Scene: transition / UI / cutscene | 132 | 40 | 48 | 0.05% | 99.54% | 12 | 4/40 (10.00%) | 0.900 | 0 | 0/40 (0.00%) | 1.200 | 0 |
| `1A/14` | module1_a_credits | Scene: polygon / cutscene | 1 | 1 | 47 | 0.05% | 99.59% | 42 | 0/1 (0.00%) | 5.000 | 0 | 0/1 (0.00%) | 47.000 | 0 |
| `1A/02` | module1_a_credits | Scene: polygon / cutscene | 1 | 1 | 46 | 0.04% | 99.63% | 41 | 0/1 (0.00%) | 5.000 | 0 | 0/1 (0.00%) | 46.000 | 0 |
| `1A/16` | module1_a_credits | Scene: polygon / cutscene | 1 | 1 | 45 | 0.04% | 99.68% | 40 | 0/1 (0.00%) | 5.000 | 0 | 0/1 (0.00%) | 45.000 | 0 |
| `07/18` | module07_18_rescued_maiden | Scene: transition / UI / cutscene | 294 | 14 | 41 | 0.04% | 99.72% | 28 | 7/14 (50.00%) | 0.929 | 0 | 0/14 (0.00%) | 2.929 | 0 |
| `1A/20` | module1_a_credits | Scene: polygon / cutscene | 1 | 1 | 41 | 0.04% | 99.76% | 32 | 0/1 (0.00%) | 9.000 | 0 | 0/1 (0.00%) | 41.000 | 0 |
| `09/12` | module09_load_new_sprites | Scene: load / upload | 16 | 16 | 32 | 0.03% | 99.79% | 0 | 0/16 (0.00%) | 2.000 | 0 | 0/16 (0.00%) | 2.000 | 0 |
| `12/08` | module12_game_over | Scene: transition / UI / cutscene | 4 | 4 | 32 | 0.03% | 99.82% | 28 | 0/4 (0.00%) | 1.000 | 0 | 0/4 (0.00%) | 8.000 | 0 |
| `0B/25` | overworld_load_overlays | Scene: load / upload | 4 | 4 | 30 | 0.03% | 99.85% | 0 | 0/4 (0.00%) | 7.500 | 0 | 0/4 (0.00%) | 7.500 | 0 |
| `0B/00` | module09_00_player_control | Gameplay load | 12,309 | 13 | 29 | 0.03% | 99.88% | 16 | 0/13 (0.00%) | 1.000 | 0 | 0/13 (0.00%) | 2.231 | 0 |
| `0B/18` | overworld_func18 | Scene: transition / UI / cutscene | 4 | 4 | 24 | 0.02% | 99.90% | 0 | 0/4 (0.00%) | 6.000 | 0 | 0/4 (0.00%) | 6.000 | 0 |
| `12/02` | module12_game_over | Scene: transition / UI / cutscene | 736 | 23 | 23 | 0.02% | 99.92% | 22 | 22/23 (95.65%) | 0.043 | 23 | 23/23 (100.00%) | 0.000 | 0 |
| `12/05` | module12_game_over | Scene: transition / UI / cutscene | 2,208 | 19 | 19 | 0.02% | 99.94% | 0 | 0/19 (0.00%) | 1.000 | 0 | 0/19 (0.00%) | 1.000 | 0 |
| `00/07` | module00_intro | Scene: polygon / cutscene | 175 | 14 | 14 | 0.01% | 99.96% | 0 | 0/14 (0.00%) | 1.000 | 0 | 0/14 (0.00%) | 1.000 | 0 |
| `15/05` | module15_mirror_warp_from_aga | Scene: transition / UI / cutscene | 1 | 1 | 14 | 0.01% | 99.97% | 14 | 1/1 (100.00%) | 0.000 | 0 | 0/1 (0.00%) | 14.000 | 0 |
| `01/02` | module01_file_select | Scene: transition / UI / cutscene | 7 | 7 | 7 | 0.01% | 99.98% | 0 | 0/7 (0.00%) | 1.000 | 0 | 0/7 (0.00%) | 1.000 | 0 |
| `00/0A` | module00_intro | Scene: polygon / cutscene | 3 | 3 | 6 | 0.01% | 99.98% | 6 | 3/3 (100.00%) | 0.000 | 0 | 0/3 (0.00%) | 2.000 | 0 |
| `12/09` | module12_game_over | Scene: transition / UI / cutscene | 720 | 4 | 4 | 0.00% | 99.99% | 0 | 0/4 (0.00%) | 1.000 | 0 | 0/4 (0.00%) | 1.000 | 0 |
| `13/04` | module13_boss_victory_pendant | Scene: transition / UI / cutscene | 3 | 3 | 3 | 0.00% | 99.99% | 3 | 3/3 (100.00%) | 0.000 | 3 | 3/3 (100.00%) | 0.000 | 0 |
| `00/02` | module00_intro | Scene: polygon / cutscene | 1 | 1 | 2 | 0.00% | 99.99% | 2 | 1/1 (100.00%) | 0.000 | 0 | 0/1 (0.00%) | 2.000 | 0 |
| `00/06` | module00_intro | Scene: polygon / cutscene | 84 | 2 | 2 | 0.00% | 99.99% | 0 | 0/2 (0.00%) | 1.000 | 0 | 0/2 (0.00%) | 1.000 | 0 |
| `04/01` | module04_name_file | Scene: transition / UI / cutscene | 2 | 2 | 2 | 0.00% | 99.99% | 0 | 0/2 (0.00%) | 1.000 | 0 | 0/2 (0.00%) | 1.000 | 0 |
| `00/00` | module00_intro | Scene: polygon / cutscene | 1 | 1 | 1 | 0.00% | 100.00% | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 | 0 |
| `03/01` | module03_kill_file | Scene: transition / UI / cutscene | 1 | 1 | 1 | 0.00% | 100.00% | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 | 0 |
| `0E/01` | hud_module_run | Scene: transition / UI / cutscene | 69,382 | 1 | 1 | 0.00% | 100.00% | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 | 0 |
| `09/06` | overworld_run_scroll_transition | Scene: transition / UI / cutscene | 7,556 | 1 | 1 | 0.00% | 100.00% | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 | 0 |
| `07/0B` | module07_0b_drain_swamp_pool | Scene: transition / UI / cutscene | 205 | 1 | 1 | 0.00% | 100.00% | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 | 0 |
| `19/01` | module19_triforce_room | Scene: polygon / cutscene | 274 | 1 | 1 | 0.00% | 100.00% | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 | 0 |

## Generic refinements by category

P1 and P2 have the formulas below; neither is fitted by category. All category metrics, including all-iteration MAE and within-one frame weights, are also retained in `target/pacing-predictor/metrics.json`.

| Entry | Actual lag NMIs | P1 frames | P1 exact lagged | P1 MAE | P2 frames | P2 exact lagged | P2 MAE |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 12,174 | 3,575/4,418 (80.92%) | 0.196 | 13,066 | 4,137/4,418 (93.64%) | 0.068 |
| `07/02` | 8,770 | 7,450 | 1,076/1,976 (54.45%) | 0.668 | 7,714 | 1,189/1,976 (60.17%) | 0.547 |
| `06/00` | 8,196 | 7,342 | 0/142 (0.00%) | 6.014 | 7,557 | 0/142 (0.00%) | 4.500 |
| `07/0E` | 7,979 | 7,649 | 4,686/4,952 (94.63%) | 0.067 | 7,729 | 4,757/4,952 (96.06%) | 0.050 |
| `08/00` | 6,466 | 5,678 | 0/157 (0.00%) | 5.019 | 5,847 | 2/157 (1.27%) | 3.943 |
| `09/23` | 6,054 | 3,710 | 484/1,179 (41.05%) | 2.047 | 3,849 | 556/1,179 (47.16%) | 2.003 |
| `0F/01` | 5,432 | 1,368 | 957/4,879 (19.61%) | 0.833 | 1,655 | 1,244/4,879 (25.50%) | 0.774 |
| `09/03` | 4,372 | 753 | 0/251 (0.00%) | 14.418 | 753 | 0/251 (0.00%) | 14.418 |
| `07/07` | 4,060 | 2,643 | 1,534/2,919 (52.55%) | 0.485 | 3,234 | 2,117/2,919 (72.52%) | 0.284 |
| `07/00` | 3,679 | 1,847 | 1,128/2,960 (38.11%) | 0.619 | 1,929 | 1,208/2,960 (40.81%) | 0.592 |
| `09/01` | 3,479 | 3,689 | 41/251 (16.33%) | 0.837 | 3,750 | 15/251 (5.98%) | 1.080 |
| `07/0F` | 2,992 | 2,022 | 1,748/2,718 (64.31%) | 0.357 | 2,142 | 1,868/2,718 (68.73%) | 0.313 |
| `10/01` | 2,672 | 231 | 228/2,669 (8.54%) | 0.915 | 386 | 383/2,669 (14.35%) | 0.857 |
| `0E/03` | 2,672 | 2,666 | 0/115 (0.00%) | 1.339 | 2,748 | 39/115 (33.91%) | 1.304 |
| `08/02` | 2,576 | 0 | 0/157 (0.00%) | 16.408 | 0 | 0/157 (0.00%) | 16.408 |
| `07/15` | 2,333 | 2,027 | 1,438/1,735 (82.88%) | 0.176 | 2,167 | 1,577/1,735 (90.89%) | 0.096 |
| `19/02` | 2,001 | 30 | 0/1,855 (0.00%) | 1.063 | 30 | 0/1,855 (0.00%) | 1.063 |
| `0E/0A` | 1,845 | 1,042 | 0/90 (0.00%) | 8.922 | 1,074 | 0/90 (0.00%) | 8.567 |
| `09/00` | 1,253 | 84 | 23/1,192 (1.93%) | 0.981 | 86 | 25/1,192 (2.10%) | 0.979 |
| `0E/07` | 1,065 | 950 | 3/56 (5.36%) | 2.946 | 979 | 0/56 (0.00%) | 3.464 |
| `07/06` | 984 | 908 | 549/603 (91.04%) | 0.126 | 935 | 572/603 (94.86%) | 0.088 |
| `11/00` | 973 | 982 | 8/17 (47.06%) | 0.529 | 1,012 | 0/17 (0.00%) | 2.294 |
| `07/01` | 928 | 916 | 916/928 (98.71%) | 0.013 | 922 | 922/928 (99.35%) | 0.006 |
| `08/01` | 706 | 0 | 0/157 (0.00%) | 4.497 | 0 | 0/157 (0.00%) | 4.497 |
| `05/00` | 639 | 533 | 0/11 (0.00%) | 9.636 | 549 | 0/11 (0.00%) | 8.182 |
| `09/04` | 508 | 0 | 0/251 (0.00%) | 2.024 | 0 | 0/251 (0.00%) | 2.024 |
| `12/03` | 472 | 10 | 0/391 (0.00%) | 1.182 | 21 | 0/391 (0.00%) | 1.153 |
| `0F/00` | 449 | 287 | 125/287 (43.55%) | 0.564 | 287 | 125/287 (43.55%) | 0.564 |
| `07/12` | 391 | 311 | 163/239 (68.20%) | 0.335 | 330 | 182/239 (76.15%) | 0.255 |
| `09/2E` | 363 | 154 | 7/100 (7.00%) | 2.170 | 162 | 10/100 (10.00%) | 2.150 |
| `07/16` | 303 | 0 | 0/101 (0.00%) | 3.000 | 0 | 0/101 (0.00%) | 3.000 |
| `09/11` | 274 | 48 | 0/16 (0.00%) | 14.125 | 48 | 0/16 (0.00%) | 14.125 |
| `09/16` | 265 | 58 | 26/233 (11.16%) | 0.888 | 81 | 49/233 (21.03%) | 0.790 |
| `09/0D` | 247 | 48 | 24/223 (10.76%) | 0.892 | 84 | 60/223 (26.91%) | 0.731 |
| `0E/0B` | 243 | 243 | 81/81 (100.00%) | 0.000 | 245 | 79/81 (97.53%) | 0.025 |
| `09/0F` | 242 | 304 | 0/16 (0.00%) | 3.875 | 319 | 0/16 (0.00%) | 4.812 |
| `01/00` | 235 | 151 | 0/4 (0.00%) | 21.000 | 155 | 0/4 (0.00%) | 20.000 |
| `09/2C` | 228 | 147 | 18/36 (50.00%) | 2.361 | 156 | 23/36 (63.89%) | 2.278 |
| `10/00` | 157 | 0 | 0/157 (0.00%) | 1.000 | 0 | 0/157 (0.00%) | 1.000 |
| `17/01` | 141 | 111 | 0/3 (0.00%) | 10.000 | 114 | 0/3 (0.00%) | 9.000 |
| `1B/02` | 139 | 135 | 5/9 (55.56%) | 0.889 | 138 | 5/9 (55.56%) | 0.556 |
| `09/20` | 138 | 0 | 0/28 (0.00%) | 4.929 | 0 | 0/28 (0.00%) | 4.929 |
| `0E/05` | 138 | 2 | 1/119 (0.84%) | 1.143 | 2 | 1/119 (0.84%) | 1.143 |
| `00/01` | 114 | 46 | 1/11 (9.09%) | 6.182 | 48 | 1/11 (9.09%) | 6.000 |
| `0B/24` | 103 | 14 | 4/93 (4.30%) | 0.957 | 17 | 7/93 (7.53%) | 0.925 |
| `09/21` | 84 | 0 | 0/28 (0.00%) | 3.000 | 0 | 0/28 (0.00%) | 3.000 |
| `09/0E` | 83 | 0 | 0/16 (0.00%) | 5.188 | 0 | 0/16 (0.00%) | 5.188 |
| `15/07` | 83 | 84 | 78/79 (98.73%) | 0.013 | 84 | 78/79 (98.73%) | 0.013 |
| `19/00` | 72 | 48 | 0/4 (0.00%) | 10.500 | 50 | 0/4 (0.00%) | 10.500 |
| `07/13` | 71 | 70 | 46/47 (97.87%) | 0.021 | 71 | 47/47 (100.00%) | 0.000 |
| `0B/17` | 68 | 6 | 3/65 (4.62%) | 0.954 | 9 | 6/65 (9.23%) | 0.908 |
| `0B/28` | 68 | 0 | 0/4 (0.00%) | 17.000 | 0 | 0/4 (0.00%) | 17.000 |
| `0B/2B` | 66 | 6 | 6/66 (9.09%) | 0.909 | 16 | 16/66 (24.24%) | 0.758 |
| `0B/19` | 64 | 0 | 0/4 (0.00%) | 16.000 | 0 | 0/4 (0.00%) | 16.000 |
| `15/03` | 64 | 40 | 8/12 (66.67%) | 2.000 | 42 | 8/12 (66.67%) | 2.000 |
| `00/05` | 62 | 0 | 0/62 (0.00%) | 1.000 | 0 | 0/62 (0.00%) | 1.000 |
| `0B/26` | 61 | 76 | 0/4 (0.00%) | 3.750 | 80 | 0/4 (0.00%) | 4.750 |
| `0B/1A` | 60 | 74 | 0/4 (0.00%) | 3.500 | 78 | 0/4 (0.00%) | 4.500 |
| `1A/12` | 60 | 39 | 0/3 (0.00%) | 7.667 | 40 | 0/3 (0.00%) | 8.000 |
| `0B/29` | 59 | 14 | 4/51 (7.84%) | 0.922 | 18 | 2/51 (3.92%) | 0.961 |
| `18/00` | 59 | 39 | 0/2 (0.00%) | 10.000 | 40 | 0/2 (0.00%) | 9.500 |
| `1A/00` | 59 | 39 | 1/3 (33.33%) | 6.667 | 40 | 0/3 (0.00%) | 7.000 |
| `1A/1A` | 59 | 38 | 0/3 (0.00%) | 7.667 | 39 | 0/3 (0.00%) | 8.000 |
| `1A/1C` | 59 | 39 | 0/3 (0.00%) | 7.333 | 40 | 0/3 (0.00%) | 7.667 |
| `1A/04` | 58 | 39 | 0/3 (0.00%) | 7.000 | 40 | 0/3 (0.00%) | 7.333 |
| `1A/08` | 58 | 38 | 0/3 (0.00%) | 7.333 | 39 | 0/3 (0.00%) | 7.667 |
| `1A/0A` | 58 | 39 | 0/3 (0.00%) | 7.667 | 40 | 0/3 (0.00%) | 8.000 |
| `1A/18` | 58 | 39 | 0/3 (0.00%) | 7.000 | 40 | 0/3 (0.00%) | 7.333 |
| `1A/0E` | 57 | 39 | 0/3 (0.00%) | 7.333 | 40 | 0/3 (0.00%) | 7.667 |
| `1A/10` | 56 | 39 | 0/3 (0.00%) | 7.000 | 40 | 0/3 (0.00%) | 7.333 |
| `1A/06` | 55 | 38 | 0/3 (0.00%) | 6.333 | 40 | 0/3 (0.00%) | 7.000 |
| `07/03` | 53 | 0 | 0/53 (0.00%) | 1.000 | 0 | 0/53 (0.00%) | 1.000 |
| `1A/0C` | 53 | 39 | 0/2 (0.00%) | 9.000 | 40 | 0/2 (0.00%) | 9.500 |
| `11/07` | 51 | 51 | 17/17 (100.00%) | 0.000 | 51 | 17/17 (100.00%) | 0.000 |
| `13/05` | 51 | 0 | 0/51 (0.00%) | 1.000 | 0 | 0/51 (0.00%) | 1.000 |
| `1A/1E` | 51 | 38 | 0/2 (0.00%) | 7.500 | 40 | 0/2 (0.00%) | 8.500 |
| `0B/1C` | 48 | 13 | 5/40 (12.50%) | 0.875 | 18 | 10/40 (25.00%) | 0.750 |
| `1A/14` | 47 | 46 | 0/1 (0.00%) | 1.000 | 47 | 1/1 (100.00%) | 0.000 |
| `1A/02` | 46 | 45 | 0/1 (0.00%) | 1.000 | 46 | 1/1 (100.00%) | 0.000 |
| `1A/16` | 45 | 44 | 0/1 (0.00%) | 1.000 | 45 | 1/1 (100.00%) | 0.000 |
| `07/18` | 41 | 28 | 7/14 (50.00%) | 0.929 | 28 | 7/14 (50.00%) | 0.929 |
| `1A/20` | 41 | 36 | 0/1 (0.00%) | 5.000 | 37 | 0/1 (0.00%) | 4.000 |
| `09/12` | 32 | 0 | 0/16 (0.00%) | 2.000 | 0 | 0/16 (0.00%) | 2.000 |
| `12/08` | 32 | 28 | 0/4 (0.00%) | 1.000 | 32 | 4/4 (100.00%) | 0.000 |
| `0B/25` | 30 | 0 | 0/4 (0.00%) | 7.500 | 0 | 0/4 (0.00%) | 7.500 |
| `0B/00` | 29 | 20 | 4/13 (30.77%) | 0.692 | 20 | 4/13 (30.77%) | 0.692 |
| `0B/18` | 24 | 0 | 0/4 (0.00%) | 6.000 | 0 | 0/4 (0.00%) | 6.000 |
| `12/02` | 23 | 22 | 22/23 (95.65%) | 0.043 | 23 | 23/23 (100.00%) | 0.000 |
| `12/05` | 19 | 0 | 0/19 (0.00%) | 1.000 | 0 | 0/19 (0.00%) | 1.000 |
| `00/07` | 14 | 0 | 0/14 (0.00%) | 1.000 | 0 | 0/14 (0.00%) | 1.000 |
| `15/05` | 14 | 15 | 0/1 (0.00%) | 1.000 | 16 | 0/1 (0.00%) | 2.000 |
| `01/02` | 7 | 0 | 0/7 (0.00%) | 1.000 | 0 | 0/7 (0.00%) | 1.000 |
| `00/0A` | 6 | 6 | 3/3 (100.00%) | 0.000 | 6 | 3/3 (100.00%) | 0.000 |
| `12/09` | 4 | 0 | 0/4 (0.00%) | 1.000 | 0 | 0/4 (0.00%) | 1.000 |
| `13/04` | 3 | 3 | 3/3 (100.00%) | 0.000 | 3 | 3/3 (100.00%) | 0.000 |
| `00/02` | 2 | 2 | 1/1 (100.00%) | 0.000 | 2 | 1/1 (100.00%) | 0.000 |
| `00/06` | 2 | 0 | 0/2 (0.00%) | 1.000 | 0 | 0/2 (0.00%) | 1.000 |
| `04/01` | 2 | 0 | 0/2 (0.00%) | 1.000 | 0 | 0/2 (0.00%) | 1.000 |
| `00/00` | 1 | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 |
| `03/01` | 1 | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 |
| `0E/01` | 1 | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 |
| `09/06` | 1 | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 |
| `07/0B` | 1 | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 |
| `19/01` | 1 | 0 | 0/1 (0.00%) | 1.000 | 0 | 0/1 (0.00%) | 1.000 |

Zero-source-lag contexts with a prediction, native hold or masked callback:

| Entry | Iterations | P0 / P1 / P2 frames | Native held | Masked elapsed |
| --- | ---: | ---: | ---: | ---: |
| `15/00` | 1 | 0 / 0 / 0 | 0 | 24 |

## Largest failures

All three rules predict zero for `08/01`, `08/02` and `09/04`, losing all **3,790** source lag NMIs in those categories. Their mean captured costs are only about 17,502, 17,508 and 91,406 master clocks, respectively. Refresh and interrupt overhead cannot make those incomplete work sums represent the omitted loops.

For the two major closing/opening iris categories (`0F/01`, `10/01`), P0 gives **1,599** holds against **8,104** actual; production gives **7,548**. Replacing existing iris pacing with this predictor would lose substantial restored duration. In contrast, `07/0E` already achieves **94.16%** exact lagged-iteration counts under P0, and P2 reaches **93.64%** in `0E/02` (4,137/4,418), with 13,066 predicted frames against 12,964 actual. These are useful category results, but they do not establish a general rule or correct mid-iteration event placement. Completing missing work prices and accounting for raster waits and CPU ownership are prerequisites to considering a general production pacing rule; no such repairs were attempted here.

Ranked by P0 absolute error across lagged iterations. This avoids confusing a very rare long miss with the largest route-wide deficit.

| Entry | Actual | P0 | P0 total absolute error on lagged calls | P0 exact lagged | P0 MAE |
| --- | ---: | ---: | ---: | ---: | ---: |
| `0F/01` | 5,432 | 1,368 | 4,064 | 957/4,879 (19.61%) | 0.833 |
| `09/03` | 4,372 | 745 | 3,627 | 0/251 (0.00%) | 14.450 |
| `0E/02` | 12,964 | 10,245 | 2,719 | 1,741/4,418 (39.41%) | 0.615 |
| `08/02` | 2,576 | 0 | 2,576 | 0/157 (0.00%) | 16.408 |
| `09/23` | 6,054 | 3,522 | 2,532 | 366/1,179 (31.04%) | 2.148 |
| `10/01` | 2,672 | 231 | 2,441 | 228/2,669 (8.54%) | 0.915 |
| `07/02` | 8,770 | 6,600 | 2,170 | 776/1,976 (39.27%) | 1.098 |
| `19/02` | 2,001 | 15 | 1,986 | 0/1,855 (0.00%) | 1.071 |
| `07/00` | 3,679 | 1,738 | 1,941 | 1,019/2,960 (34.43%) | 0.656 |
| `07/07` | 4,060 | 2,550 | 1,510 | 1,523/2,919 (52.18%) | 0.517 |
| `09/00` | 1,253 | 75 | 1,178 | 14/1,192 (1.17%) | 0.988 |
| `07/0F` | 2,992 | 2,021 | 971 | 1,747/2,718 (64.28%) | 0.357 |

Largest individual P0 errors (iteration indexes are zero-based):

| Iteration | Entry | Actual | P0 | Absolute error | Main master clocks | Preceding NMI clocks |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 8 | `00/01` | 48 | 0 | 48 | 17,412 | 8,964 |
| 629,840 | `08/00` | 65 | 39 | 26 | 14,048,518 | 37,540 |
| 884,324 | `08/02` | 25 | 0 | 25 | 17,418 | 90,456 |
| 1,320,897 | `08/02` | 24 | 0 | 24 | 17,412 | 90,456 |
| 1,066,960 | `08/02` | 24 | 0 | 24 | 17,418 | 90,456 |
| 1,066,958 | `08/00` | 59 | 35 | 24 | 12,798,936 | 37,494 |
| 898,078 | `01/00` | 60 | 36 | 24 | 13,053,214 | 25,404 |
| 884,322 | `08/00` | 59 | 35 | 24 | 12,793,450 | 36,050 |
| 738,185 | `08/02` | 24 | 0 | 24 | 17,418 | 90,456 |
| 738,183 | `08/00` | 59 | 35 | 24 | 12,817,574 | 37,540 |
| 696,237 | `01/00` | 60 | 36 | 24 | 13,010,986 | 25,404 |
| 465,811 | `08/00` | 59 | 35 | 24 | 12,790,476 | 36,004 |

P0 predicts zero on 16,888 lagged iterations. The captured main cost is zero on 0 iterations; a nonzero dispatcher/helper charge does not establish that expensive child work was charged. The preceding handler estimate is unavailable on one iteration (recorded as zero); the defensive handler cap is reached 0 times.

## What was measured

This is a diagnostic on `codex/pacing-predictor` at `b47505a0`. No production pacing changes were made. The only runtime-library edit exposes the existing `lag_free_main_master()` reader; the logical-replay harness captures it immediately after the first callback, before draining production iris/song callbacks. `lag_free_last_main_master()` is **not** used: it is set only on polygon-thread paths and is stale elsewhere. The captured cost is the existing frame-start-to-NMI-acceptance ledger delta plus `lag_free_vwf_master`, with a current-ledger fallback when the callback masks NMI. Closing NMI CPU/DMA cost and subsequent held callbacks are excluded from main cost. Polygon worker slices run ledger-muted. This is charged native work, not a measured cartridge main duration.

Let `F = 1364 * 262 = 357368`, `N = 225 * 1364 = 306900`, `C = main_master`, and `H` be the preceding native NMI handler's existing CPU+DMA cost (zero if unavailable). Main enters at `N + H`; following NMI points are `N + kF`. The primary prediction **P0** is `floor((C + H) / F)`. Thus line 225 cancels when measuring from the preceding NMI. The implementation bounds `H` to `F-1` to keep the generic budget positive. A crossing at equality is counted; no per-scene thresholds or fitted multipliers are used.

Only two refinements were evaluated, in the same full pass:

- **P1, interrupt overhead:** `floor(C / (F-H))`. Reuse the preceding handler estimate at every intervening NMI, including its DMA. This is a generic estimate: an intervening lag handler can do less work than the preceding update handler.
- **P2, refresh:** `floor((C + 40*ceil(C/1324)) / (F-H))`. Add a coarse 40-clock WRAM-refresh stall for each 1324 clocks of CPU work. This does not model actual refresh phase, HDMA, V-IRQ worker ownership, masked-NMI windows, or different intervening handler costs.

All predictions use native counters only. Ledger `lag_nmis` is read solely by the census for comparison. Predictions never call the pacing scheduler, affect exception classification, or insert callbacks. `--pacing-predictor PATH` writes one CSV row per iteration with opening context, source lag, native held/masked counts, main/VWF/preceding-NMI costs, existing silent-probe count, and P0/P1/P2. The outcome trace also exposes the predictions.

## Missing and understated charges

The following is a source audit, not an assignment of source lag to individual children: the ledger has no per-iteration cartridge main PC profile. A dispatcher without a self charge can still call charged children. `source-audit.json` retains 34 named routines, locations, direct charge/scope presence and immediate `self` callees. It is deliberately not interpreted as an inclusive-cost profiler.

| Categories | Missing or understated native work | Evidence and limit |
| --- | --- | --- |
| `08/01..02`, `09/03`, shared mirror/overlay paths (`09/23`, `09/20`, `0B/25`, etc.) | Map decompression, quadrant definition expansion and map16-to-map8 conversion have no direct cycle-ledger price | `overworld.rs`: `decompress_bank02_into` (5592), `decompress_bank02` (5668), `advance_overworld_quadrant_cursor` (5694), `draw_overworld_quadrant_definitions` (5745), `overworld_parse_map32_definition` (5773), `Map16ToMap8` (5862), `advance_overworld_map8_cursor` (5878). The decode/store loops and their semantic setters have no ROM instruction charge. These are distinct from graphics-sheet decompression. |
| `08/00`, `09/01`, `09/03..04`, `06/00` | Uncharged dispatcher/prefix/suffix work around substantially charged graphics children | `pre_overworld_load_properties` (overworld.rs:177), `pre_overworld_load_overlays` (912), `overworld_load_and_build_screen` (1087), `module09_load_aux_gfx` (655), `module09_load_new_map_and_gfx` (5959), `module09_load_new_sprites` (6202), `module_pre_dungeon` (dungeon/entrance.rs:410) have no own charge. Do **not** call their entire inclusive cost zero: `LoadTransAuxGFX`, sprite/background decompression and 3-to-4 plane conversion are charged. `PrepTransAuxGfx` itself has no self charge but calls charged converters. |
| `06/00`, `07/02`, `07/0E`, falling/warp callers | Room loader parent work and some native object/stair routines lack self charges | `dungeon_load_room` and `dungeon_load_room_post_object_prefix` (dungeon/room_load.rs:315,633), `LoadType1ObjectSubtype1` (dungeon/objects.rs:8), `room_draw_1x3_rightwards`, `object_draw_5x4`, `draw_objects_pushable_block` (dungeon/object_draw.rs:92,104,1263), `module07_0e_spiral_stairs`, `HandleLinkOnSpiralStairs` (dungeon/stairs.rs:111,1822). Object decode/dispatch and many drawing helpers **are** priced, as is `module07_02_supertile_transition`'s own dispatch. These gaps understate cost; they do not establish that every missing frame comes from them. |
| `0E/02` | Message pixel-scroll copy loop has no cycle-ledger charge; fresh sprite/lamp prefix and some command costs remain incomplete | `render_text_scroll_pixels` (messaging.rs:9580) copies seven words per cell plus line clearing per pixel, without charging those loops. The VWF glyph renderer instead contributes its separate `lag_free_vwf_master` counter (8396), which is included once. The fresh-prefix limitation is documented at 7842–7852; choice-menu commands remain unpriced at 8325. `RenderText` dispatch itself is charged. |
| `0E/03` and map/menu callers | Map dispatcher and room border/row drawing loops have no own charge | `module0e_03_dungeon_map`, `dungeon_map_draw_border_for_rooms`, `dungeon_map_draw_single_row_of_rooms` (messaging.rs:4949,5232,5301). Charged graphics decompression children can still dominate an iteration. |
| Iris (`0F/00..01`, `10/00..01`, `07/0F`, shared game-over/pendant) | Raster-dependent V-counter spin is intentionally understated | `complete_iris_spotlight_table_projection` (load_gfx.rs:6014, especially 6023–6034) charges only the final 172-clock wait-loop pass. Each omitted looping pass costs 178 clocks; the existing comment records approximately 1–950 source passes. Row/circle generation and table copy **are** priced. A work-only sum cannot recover this wait from the existing charge alone. Production iris already has a separate coarse work/raster hold rule. |
| Polygon-active contexts (overlaps the table) | Main's cost does not account for lost CPU ownership while the V-IRQ worker runs | `lag_free_run_poly_thread_phase` (rtl_poly.rs:231–280) deliberately mutes worker charges. This is appropriate for a **main-work** counter, but a general pacing budget would need to subtract the worker's share of the field. P0/P1/P2 do not do that. |
| Ordinary gameplay (`07/00`, `09/00`, `0B/00`) | Sprite/control work is only partially annotated | Existing `probe_annotation` detects wholly silent probed calls; an unpriced caller with any charged child passes that heuristic. Zero silent probes therefore do not certify complete cost. No exhaustive sprite-child cost or missing-cycle attribution is claimed. |

Song-bank elapsed time is separate: native masked callbacks are reported as `native_elapsed`, never added to accepted `lag_nmis` or treated as predicted lag. The existing uploader's cost/phase model is unchanged. These measurements do not test elapsed masked-window accuracy.


## Verification and reproduction

Retained evidence is under `target/pacing-predictor/`: `build-{default,reduced}.log`, both binaries, `census.csv`, `census.log`, `census.stderr`, `census-summary.txt`, `metrics.json`, `source-audit.json`, `verification.json` and `artifact-hashes.json`. Scratch analysis programs remain under `/tmp`; no one-off script was added to the repository.

```sh
cargo build --profile parity -p zelda3-bin --no-default-features --features parity-debug
cp target/parity/zelda3 target/pacing-predictor/reduced-zelda3
LEDGER=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
ZELDA3_SAVE_DIR=/tmp/pacing-predictor-repro target/pacing-predictor/reduced-zelda3 \
  --logical-replay "$LEDGER" "$C/initial.srm" --report 0 --seed-every 2000 \
  --pacing-predictor target/pacing-predictor/census.csv \
  > target/pacing-predictor/census.log 2> target/pacing-predictor/census.stderr
python3 scripts/logical_census_summary.py target/pacing-predictor/census.log \
  --gate --expect-iterations 1473123
cargo build --profile parity -p zelda3-bin
git diff --check
```

The full census and terminal gate both exit 0. This retains the census’s existing periodic-seed, cartridge RNG/APU-answer and classified-difference contract. PASS means no unexplained logical-state/RNG/command differences; it does not mean the predictions pass a pacing-accuracy gate or prove unseeded quest, video or exact audio timing. No GPU run/test, `--video`, cached-A/V or Snes9x run, `cargo fmt`, commit or push was performed. The supplied ledger/cache and other checkout were read-only.
