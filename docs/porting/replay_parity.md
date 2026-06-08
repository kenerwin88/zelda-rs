# Replay Parity Workflow

Use replay traces to find the first bad state before comparing source. Source
diffs are review aids; they are not the primary locator for gameplay drift.

## Standard Regression Gate

Before committing replay parity work, run the C/R standard route gate:

```sh
python3 scripts/test_standard_replay_parity.py
```

This uses `../zelda3` as the C oracle by default and
checks the canonical `saves/zelda3-combined-route.sav` route across the
multi-window regression set plus the full reset-start final frame `1073092`.
It compares RAM/SRAM hashes, HP/state, speed/line speed, RNG, room death masks,
room history masks, and route/module state.

## Fast Loop

1. Start with one known matching frame and one known divergent frame.
2. Run `scripts/replay_bisect.py` to find the first divergent checkpoint:

   ```sh
   scripts/replay_bisect.py \
     --good 204413 \
     --bad 216606 \
     --save saves/zelda3-combined-route.sav \
     --rom /path/to/zelda3.sfc
   ```

3. If the checkpoint diff is not enough, rerun with subsystem dumps:

   ```sh
   scripts/replay_bisect.py --good 204413 --bad 216606 --dumps
   ```

4. Identify the first divergent state variable, then trace the C function that
   writes it.
5. Use CodeGraph to jump to the Rust equivalent and patch for parity.

## Hidden State

Every replay checkpoint includes hidden state that commonly causes later visible
drift:

- `rng`: byte `0x0fa1`, the RNG seed.
- `roommask`: the current dungeon room's `sprite_where_in_room` death mask.
- `hist`: the four-entry dungeon room history.
- `histmask`: the death masks for those history rooms.

When sprite behavior diverges, enable focused dumps instead of broad logging:

- `ZELDA3_REPLAY_RNG_DUMP=1`
- `ZELDA3_REPLAY_ROOM_MASK_DUMP=1`
- `ZELDA3_REPLAY_ROOM_HISTORY_DUMP=1`
- `ZELDA3_REPLAY_SPRITE_DUMP=1`
- `ZELDA3_REPLAY_ANCILLA_DUMP=1`
- `ZELDA3_REPLAY_OVERLORD_DUMP=1`
- `ZELDA3_REPLAY_DOOR_DUMP=1`

## Wider-Than-Byte RAM Arrays

C pointer casts encode element width. Rust ports should avoid raw byte indexing
for those arrays. Add helper APIs for word arrays, then use tests that cover
bits above `0x00ff`. The `sprite_where_in_room` mask is a `uint16` array in C,
so Rust code must use `sprite_where_in_room_mask(room)` and
`set_sprite_where_in_room_mask(room, mask)`.

## Source Comparison

Use semantic source comparison after the trace says which function matters.
`diffoscope` is useful for generated artifacts, savestates, and binary assets.
It is not a substitute for finding the first bad replay frame because source
that looks equivalent can still be fed different hidden state.
