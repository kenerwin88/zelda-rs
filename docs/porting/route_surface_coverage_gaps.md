# Route Surface Coverage Gaps

Generated from the standard replay route, promoted supplemental coverage
branches, source-seeded room/screen inventory, and targeted route-surface
probes on 2026-06-22.

Coverage is a route-surface inventory, not a full parity proof by itself. A
green route fingerprint check proves behavior only for the route that was
fingerprinted. The source-seeded supplement closes the coverage inventory for
frame-sampled C room/screen assets. The targeted probe supplements add normal
`first_seen` route-evidence frames for source-backed rooms/screens that broad
joystick sweeps failed to traverse naturally.

## Current Gate

```sh
./target/debug/zparity coverage \
  --from-json .cache/parity-probes/coverage-full-first-seen.json \
  --from-json .cache/parity-golden/coverage-branch-357697-main-menu-modules.json \
  --from-json .cache/parity-golden/coverage-stop-after-load-no-input-30000.json \
  --from-json .cache/parity-golden/coverage-branch-89424-bush-poof-ancilla.json \
  --from-json .cache/parity-golden/coverage-branch-916218-somaria-fission-ancilla.json \
  --from-json .cache/parity-golden/coverage-branch-511196-archery-game.json \
  --from-json .cache/parity-golden/coverage-branch-496311-pikit-shield-drop.json \
  --from-json .cache/parity-golden/coverage-branch-872500-flute-stop4-ow2f.json \
  --from-json .cache/parity-golden/coverage-branch-591900-wilted-terrace-ow7c.json \
  --from-json .cache/parity-golden/coverage-source-seeded-c-assets.json \
  --json .cache/parity-golden/coverage-merged-source-seeded.json \
  --report-json .cache/parity-golden/coverage-report-source-seeded.json \
  --require-full
```

This source-surface command is green. Without the source-seeded supplement, the
organic route suite still misses 66 indoor rooms and 4 overworld screens.

## Route-Evidence Gate

```sh
./target/debug/zparity coverage \
  --from-json .cache/parity-golden/coverage-merged-route-probed.json \
  --route-report-json .cache/parity-golden/coverage-route-evidence-report.json \
  --route-worklist-json .cache/parity-golden/coverage-route-worklist.json \
  --require-route-full
```

This command is green after merging targeted probe supplements. Source-seeded
provenance alone does not satisfy `--require-route-full`; the route-evidence
gate requires replay or probe logs with `first_seen` frames.

The generated route worklist is currently empty:

```text
.cache/parity-golden/coverage-route-worklist.json
```

Probe supplement:

- `coverage-route-probes-worklist-all.json`: generated from
  `coverage-route-worklist-source-seeded.json`; runs 28 direct entrance probes,
  43 focused dungeon-room probes, and 4 overworld screen probes.

## Source-Surface Summary

| Surface | Hit | Expected | Coverage | Missed |
|---|---:|---:|---:|---:|
| `main_modules` | 25 | 25 | 100.0% | 0 |
| `module_states` | 1494 | 1494 | 100.0% | 0 |
| `sprite_types` | 232 | 232 | 100.0% | 0 |
| `ancilla_types` | 60 | 60 | 100.0% | 0 |
| `indoor_rooms` | 296 | 296 | 100.0% | 0 |
| `overworld_screens` | 83 | 83 | 100.0% | 0 |
| `active_items` | 20 | 20 | 100.0% | 0 |

Promoted branches now close the previously blocking sprite, ancilla, active
item, and two overworld misses:

- `coverage-branch-511196-archery-game` covers sprite `0x65` and room
  `0x0111`.
- `coverage-branch-496311-pikit-shield-drop` covers sprite `0xe6` by killing a
  Pikit while `sprite_E == 0x04`.
- `coverage-branch-89424-bush-poof-ancilla` covers ancilla `0x3f`.
- `coverage-branch-916218-somaria-fission-ancilla` covers ancilla `0x2e`.
- `coverage-branch-872500-flute-stop4-ow2f` covers overworld screen `0x002f`.
- `coverage-branch-591900-wilted-terrace-ow7c` covers overworld screen
  `0x007c`.
- `coverage-source-seeded-c-assets` covers the remaining 66 indoor rooms and 4
  overworld screens from C asset filenames, with raw-log provenance entries
  such as `source-seeded:assets/dungeon/dungeon-272.yaml`.

## Route-Evidence Summary

| Surface | Hit | Expected | Coverage | Missed |
|---|---:|---:|---:|---:|
| `main_modules` | 25 | 25 | 100.0% | 0 |
| `module_states` | 1495 | 1495 | 100.0% | 0 |
| `sprite_types` | 232 | 232 | 100.0% | 0 |
| `ancilla_types` | 60 | 60 | 100.0% | 0 |
| `indoor_rooms` | 296 | 296 | 100.0% | 0 |
| `overworld_screens` | 83 | 83 | 100.0% | 0 |
| `active_items` | 20 | 20 | 100.0% | 0 |

## Source-Seeded Indoor Rooms

These are no longer missed by the full source-surface coverage gate or the
targeted route-evidence gate, but some still lack natural traversal proof from
the organic replay route.

All required indoor room IDs correspond to C asset files under
`assets/dungeon/dungeon-*.yaml`. They should remain required unless a narrower
source audit proves a specific asset is a placeholder that cannot become
`dungeon_room_index2` at runtime. The current route worklist is empty; use the
generated JSON artifacts rather than a handwritten room list when reviewing
future gaps.

The generated payload files `dungeon-296.yaml` through `dungeon-319.yaml`
(`0x0128..0x013f`) are intentionally excluded from this frame-sampled route
coverage universe. The coverage universe is capped at
`MAX_FRAME_SAMPLED_INDOOR_ROOM = 0x0127`, matching the static standard universe
used before asset discovery and avoiding over-counting payloads that are not
currently proven stable `dungeon_room_index2` frame values.

The old organic-route miss set is now closed by
`coverage-route-probes-worklist-all.json`, which is generated from the
source-seeded route worklist instead of a handwritten room or screen list.

## Probe Notes

First-seen neighboring-screen checkpoints are often saved mid-transition and
are poor branch starts. A broad sweep of direction holds from those checkpoints
did not add screens.

Input script frame ranges depend on the checkpoint origin:

- Checkpoints saved from the original route via `--save-state-at` retain the
  original route frame counter, so post-load scripts need absolute route frame
  ranges.
- Derived checkpoints created by loading a checkpoint and then saving a new
  state were observed to use a local post-load frame base, so short branch
  scripts should begin at frame `1`.

Use `zparity coverage --input-script-overlay` for source-backed branch searches
from replay checkpoints. It keeps consuming the original replay stream and only
substitutes input on explicit script frames, so omitted frames no longer clear
the route seed or base replay input. Reserve `--input-script` plus
`--stop-replay-after-load` for deliberate full replacement after a checkpoint or
SRAM start.

Recent scratch searches:

- `0x007f`: 373 absolute-frame whirlpool probes from the existing `0x0055`
  checkpoints produced no new coverage before the sweep was stopped.
- `0x0077`: 165 corrected short probes from derived `0x0075`, `0x007a`,
  `0x0074`, and `0x0072` checkpoints produced no new coverage.
- `0x0062` / Hyrule Castle East: a route-preserving replay-tail generator
  decoded `saves/zelda3-combined-route.sav` and reproduced frames `4000..9000`
  from `.cache/parity-probes/entrance-routes/route-004000-ow001b-stable.sav`.
  A 480-variant sweep over one-window movement overrides toward the source
  entrance on overworld screen `0x001b` produced no new coverage. That sweep can
  now be rerun without a generated base-tail script by passing the candidate
  movement windows through `--input-script-overlay`.
- Overlay reruns with replay input preserved produced no target-screen coverage
  for simple direction-hold searches:
  - `0x007f`: 63 whirlpool probes from `0x0055` checkpoints.
  - `0x0077`: 75 neighboring-screen probes from `0x0075`, `0x007a`,
    `0x0074`, `0x0072`, and `0x007b` checkpoints.
  - `0x005a`: 60 neighboring-screen probes from `0x005b` checkpoints.
  - `0x006f`: 96 neighboring-screen probes from `0x006e` and `0x0070`
    checkpoints.

## Recommended Next Work

1. Use `coverage-merged-route-probed.json` with `--require-route-full` as the
   current 100% automated route-surface gate.
2. Keep the organic-route gap distinction visible in docs and reviews: the
   targeted probes prove the coverage recorder and source-backed surface IDs,
   not full natural traversal inputs for every room.
3. Stop broad neighboring-screen joystick sweeps for these misses. The targeted
   probes close the actionable coverage gap far faster and leave a reproducible
   JSON trail.
4. Only add future exclusions when C source proves the runtime RAM value cannot
   be produced. Empty names in YAML are not enough; many unnamed rooms still
   have real headers, stairs, doors, layers, chests, or travel destinations.
