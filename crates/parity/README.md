# zparity — per-frame C-oracle parity checker for zelda3-rs

`zparity` is a C-free parity checker that surfaces per-frame WRAM/VRAM/SRAM/render
divergences between the Rust port and the C oracle. The workflow is:

```
capture (C oracle, once)  →  golden committed to parity-golden/
check   (C-free, sharded) →  DIVERGE at frame N  or  MATCH
drill   (C-free)          →  per-page/layer localization for frame N
```

## Subcommands

### `zparity capture [--full] [--detail]`

Runs the C oracle over the full replay route and writes the two-tier golden:

- **Tier A** (`parity-golden/rollup.bin`, `merkle.bin`, `manifest.json`) — per-block
  rollup fingerprints committed to the repo. Small (~4.3 MB for the full route).
- **Tier B** (`.cache/parity-golden/detail/`) — per-frame per-region fingerprints,
  written only with `--detail`. Large (several GB). Cached locally, not committed.
  Needed by `drill`.

`--full` runs the full ~1,073,092-frame route (default: 30,000 frames).
`--detail` also writes Tier B to `.cache/parity-golden/detail/` (gitignored).

Re-capture when: the replay route changes, a C-oracle hook is updated, or the
fingerprint mask changes.

### `zparity check [--full] [--frames N]`

**C-free.** Shards the route across CPU cores using checkpoints seeded from the
Rust binary. Compares per-block fingerprints against the committed Tier A golden
(`parity-golden/rollup.bin`).

Output:
- `DIVERGE at frame N, page P (WRAM/VRAM/SRAM/RENDER)  rust=0x...  golden=0x...` — first
  per-frame divergence found.
- `MATCH  K frames, S shards, Ts  root=0x...` — all frames match the golden.

**`check` is a STRICT per-frame-vs-C bug-finder and is currently RED by design.**
The Rust port has known per-frame divergences that are real bugs to fix
(first one: frame 3739, WRAM page 3, `RAW_SFX_PAN_VALUE`, rust=0x0f vs C=0x20).
`check` is a *worklist tool*: run `check` to find the earliest divergence, run
`drill <frame>` to localize it, fix the Rust side, repeat. It is intentionally
**NOT** wired as a blocking pre-commit gate (that would block all commits until
every divergence is fixed).

Measured speed: ~30,000 frames in ~69 seconds across 4 shards (incl. checkpoint seeding).
Full-route check time is proportionally longer.

### `zparity drill <frame>`

**C-free.** Localizes the divergence at frame `<frame>` per page and per layer
(WRAM/VRAM/SRAM/RENDER). Requires Tier B detail (`capture --full --detail`).

Output: table of diverging pages with golden vs rust fingerprints.

## Two-tier golden layout

```
parity-golden/
  manifest.json      # schema, frames, ROM sha256, save sha256, C-oracle rev, timing-hacks, mask, block_size
  rollup.bin         # Tier A: per-block rollup fingerprints (~4.3 MB full route)
  merkle.bin         # Tier A: merkle root over rollup blocks (28 bytes)
  .gitignore         # excludes detail/ and *.fp (Tier B)

.cache/parity-golden/
  detail/            # Tier B: per-frame per-region fingerprints (large; local only)
```

Tier A is committed. Tier B is local-only (`.cache/parity-golden/` is gitignored).

## Environment overrides

| Variable | Default | Description |
|---|---|---|
| `ZELDA3_C_REPO` | `../zelda3` | Path to the C oracle repo (used by `capture`) |
| `ZELDA3_NEW_BIN` | `target/parity/zelda3` | Rust binary (used by `check` for checkpoint seeding) |
| `ZELDA3_ROM` | `saves/zelda3.sfc` | ROM file |
| `ZELDA3_REPLAY_SAVE` | `saves/zelda3-combined-route.sav` | Replay save |

## Workflow

### Finding and fixing a divergence

```bash
# 1. Find the earliest diverging frame (fast, C-free):
./target/debug/zparity check --frames 50000

# 2. Localize per page/layer (needs Tier B; capture once with --detail):
./target/debug/zparity capture --full --detail    # one-time; takes 10-30+ min
./target/debug/zparity drill 3739

# 3. Fix the Rust-side bug (see CLAUDE.md debugging loop).

# 4. Verify the fix pushed the divergence later:
./target/debug/zparity check --frames 50000
```

### Re-capturing after a route or C-oracle change

```bash
make -C ../zelda3 zelda3
cargo build --profile parity -p zelda3-bin
cargo build -p parity
./target/debug/zparity capture --full          # Tier A only; commit parity-golden/
# (Optional) also capture Tier B for drill:
./target/debug/zparity capture --full --detail
```

### Spec and plan

- Spec: `.git/sdd/spec.md`
- Implementation plan: `.git/sdd/plan.md`
- Task briefs and reports: `.git/sdd/task-*-{brief,report}.md`
