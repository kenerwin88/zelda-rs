# Workaround cleanup audit — 2026-09-07

Baseline: `a44598b3d93c3febab7fa3003a758f89284c40d7`.
Oracle: the pinned full-run cache
`ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18`.

## Removed: obsolete room 0x72 publication overrides

`Dungeon_InterRoomTrans_State9` and `Dungeon_InterRoomTrans_State10`
in `crates/zelda3/src/dungeon.rs` forced an OBJ scanout selection, set the
NMI-update latch, and disabled core updates for room `0x72`. State 9 also
serves transition state 11. These branches dated from interrupted-quadrant
handling that has since been replaced by the general timing owners.

The quadrant builder already queues the core DMA; the ordinary NMI and
common-suffix paths own update acceptance and latch clearing. Removing the
two branches lets those existing owners govern this room too. It adds no new
room, frame, tile, or timing exception.

The affected transition is in take 0005 around local frames 11533–11536,
continuous source host calls 23268–23271. Native debug host H consumes
source host H-1, verified against receipt-install order. Before/after runs
captured source calls 23260–23285 and compared:

| Evidence | Result |
| --- | --- |
| Complete WRAM images | All 26 byte-identical |
| Ordered NMI entry/exit, OAM DMA and receipts, register publication, OBJ diagnostics | Identical |
| General-owner latch operations | Identical; three redundant room-specific latch sets removed |
| Full OAM at intermediate sample and composition completion | Two bytes corrected to the source values at call 23272; no new source mismatches |
| Final native OAM, after general DMA/law selection and before source override | All 26 × 544 bytes identical before/after and identical to source |
| Instrumented from-zero A/V runs | Both matched all 25,000 frames |
| Uninstrumented removal candidate | Matched all 200,000 A/V frames; endpoint WRAM matched the original baseline; both available WRAM goldens matched |

The intermediate scanout selection changes on three frames. This is not
merely a diagnostic label change: two intermediate bytes change from
205 to 207 and 209 to 211. Both corrections match the source. The remaining
23 intermediate byte mismatches exist in both builds and are resolved by
the existing general post-composition DMA/law selection. Final native OAM
has zero mismatches in this window, independently of the source override.
This bounded result does not establish that every scanout policy elsewhere
can be deleted.

Temporary full-OAM probes were removed after comparison. Local investigation
artifacts remain under `target/cleanup-multi-area/`: the two
`room72-full-*-av` runs, frozen source boundary evidence, and
`room72-full-oam-comparison.json`. These ignored artifacts are supporting
diagnostics; the committed full-route promotion receipt is the acceptance
record, linked from `routes/full_run/parity-frontier.json`.

## Removed: constant-false initial OAM deferral

In `crates/zelda3/src/nmi.rs`, `defer_intro_initialization_oam_dma` was always
false. Its negated conditional always performed the DMA. The call is now
unconditional, with its existing source-order explanation retained.
No transfer, timing boundary, or byte changes.

## Retained: room 0x22 filtered-transition fallback

The room `0x22`, substate 14 branch in `crates/zelda3/src/zelda_rtl.rs`
selects `InterruptedInModule`. The fallback in
`zelda_rtl/rtl_checkpoint.rs` can hold a whole NMI before module work when
a live main-loop return timeline is unavailable. Timeline receipts bypass
it, but the fallback has not been proven unreachable.

The underlying gap is a partial timing continuation. `Module07_02_FadedFilter`
in `dungeon.rs` explicitly rejects interruption inside the completing first
palette pass; the ordinary leading-NMI dispatcher covers substates 4–7,
not this state. Deleting the room guard or broadening it to every state-14
transition would not implement that missing behavior.

The next change should first capture an actual fallback hit with source PC,
NMI acceptance and completed palette writes. Model the interruption phase
(before module work, inside either palette walk, after SpriteMain, inside
Link OAM, or inside NMI sprite preparation), retain writes already executed,
and resume only the unfinished work. Useful source boundaries are dispatcher
`02:87AC`, SpriteMain `06:8328`, Link OAM `0D:A18E`, and NMI sprite preparation
`00:85FC`. Host 85259 is an existing state-14 control, not a confirmed fallback
reproducer. This cleanup leaves that behavior intact.
