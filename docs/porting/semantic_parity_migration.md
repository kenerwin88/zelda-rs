# Semantic Parity Migration Plan

This plan describes how to move `zelda3-rs` toward typed, readable game state while still proving that it behaves like the C reference checkout at `../zelda3`.

The short version: do not remove byte parity first. Keep byte parity as the strongest oracle while adding a semantic parity layer beside it. Once a subsystem has semantic snapshots, output parity, and route coverage that prove the same behavior, that subsystem can be refactored away from raw WRAM addresses behind a typed API.

## Goals

- Preserve the C checkout as the behavior oracle.
- Let Rust internals become clearer than raw `ram[ADDRESS]` and address-named constants.
- Avoid making independent typed fields for memory slots that are intentionally aliased in the original.
- Build semantic checks that can survive layout changes, typed state, and eventual removal of some byte-for-byte RAM storage.
- Keep visual, audio, and PPU-facing output parity as separate gates from gameplay state parity.

## Non-Goals

- Do not globally drop WRAM byte parity in one step.
- Do not claim semantic parity from visual similarity alone.
- Do not replace the existing lockstep harness with a parallel oracle runner.
- Do not force typed state into shared scratch registers until the surrounding call sites prove non-aliasing.

## Current State

The current lockstep oracle lives in `crates/zelda3/src/zelda_cpu_infra.rs`.

It compares a raw `Snapshot`:

- CPU registers for the SNES oracle side.
- WRAM bytes.
- SRAM bytes.
- VRAM words.
- CGRAM words.
- OAM words.
- Selected PPU-visible register state.

`LockstepOracle::run_frame_with_compare` currently checks byte parity before and after each frame. `zelda3-bin --lockstep` drives that path, and `scripts/oracle_windows.py` is the machine-readable route ledger runner.

This is still the right foundation. It catches all observable state drift cheaply and produces precise first-difference offsets. The problem is that byte parity also couples the Rust code to C/WRAM layout, making cleanup harder.

## Target Architecture

The target architecture has four layers.

### 1. Byte Parity Layer

Keep the existing `Snapshot` comparison as the default strict gate.

Responsibilities:

- Prove current behavior has not drifted.
- Locate first divergence precisely.
- Validate migration scaffolding while it is being built.

This layer remains mandatory until a subsystem is explicitly graduated.

### 2. Semantic Snapshot Layer

Add a typed snapshot derived from either raw WRAM/PPU state or future typed Rust state.

The snapshot should be grouped by behavioral surface:

- `FrameControl`: frame count, main module, submodule, sub-submodule, NMI routine, input-derived control state.
- `Player`: position, velocity, direction, action state, handler state, auxiliary state, item in hand, pit/fall state, health/magic/status.
- `Camera`: BG scroll, camera scroll, room/area index, indoor/outdoor mode.
- `Dungeon`: room index, floor, door state, tile update state, dungeon flags, water/HDMA state.
- `Overworld`: area index, screen index, scroll direction, Map16 load source/destination/y unit, overlay state.
- `Sprites`: active slots with type, state, position, floor, timers, health, subtype, OAM priority-relevant state.
- `Ancilla`: projectile/effect slots with type, position, timers, direction, draw-relevant state.
- `Inventory`: visible inventory, equipped item, rupees, keys, bombs, arrows, save-visible flags.
- `PpuOutput`: PPU mode, scroll, windows, brightness, screen masks, VRAM update packet state.
- `RenderOutput`: frame hash or region hashes when direct render comparison is required.
- `AudioOutput`: high-level music/SFX request state first, DSP/audio sample parity later.

The first implementation should not attempt all fields. It should start with high-value fields already used in replay diagnostics:

- module/submodule/sub-submodule
- Link position and velocity
- room/area/screen
- camera/BG scroll
- RNG
- selected item/item in hand
- health
- sprite and ancilla active-slot summaries
- PPU mode/brightness/screen masks

### 3. Typed Access Layer

Introduce typed wrappers over the existing RAM backing.

Examples:

```rust
struct PlayerStateView<'a> {
    ram: &'a mut [u8],
}

impl PlayerStateView<'_> {
    fn position(&self) -> Point3;
    fn set_position(&mut self, value: Point3);
    fn pit_correction_active(&self) -> bool;
}
```

This step improves call sites without changing storage. It also keeps aliasing visible because the wrappers still use the same underlying bytes.

Only after a subsystem has enough semantic coverage should it move from a view over RAM to owned typed state.

### 4. Typed Owned State Layer

Move selected subsystems to real typed fields when all of these are true:

- The field is not shared scratch or mode-dependent aliasing.
- Semantic snapshots cover the behavior that depends on it.
- Output-facing state still matches the C oracle.
- A bridge can serialize/deserialize to the old WRAM layout for checkpoints, tooling, and remaining C-aligned comparisons.

## Migration Phases

### Phase 0: Document the Contract

Deliverables:

- This plan.
- A short doc section explaining which parity layer proves which claim.
- A list of subsystem graduation requirements.

Verification:

- Docs are committed with the same commands used for lockstep work.

### Phase 1: Add Semantic Snapshot Scaffolding

Deliverables:

- `SemanticSnapshot` extracted from the existing raw `Snapshot`.
- `SemanticDifference` / `SemanticComparisonReport`.
- Unit tests proving:
  - identical synced states have no semantic differences.
  - changed player/module/camera fields produce named semantic diffs.
  - semantic snapshot extraction works from both SNES oracle state and Rust game state.
- CLI support to print semantic summaries during lockstep debugging without disabling byte parity.

Verification:

- `cargo test -p zelda3 semantic`
- `cargo check -p zelda3-bin`
- One existing oracle window still passes byte parity.

### Phase 2: Add Semantic-Only Diagnostics

Deliverables:

- `--trace-semantic-state` or equivalent CLI mode for `--lockstep`.
- On byte divergence, print the semantic diff first when it exists, then the raw byte diff.
- Route logs should include semantic fields with stable names instead of only addresses.

Verification:

- Existing byte parity failures still fail.
- Diagnostic output remains deterministic.
- No existing `oracle_windows.py --run` behavior changes unless the flag is requested.

### Phase 3: Build Typed Views Over RAM

Deliverables:

- Views for player, camera, module, overworld Map16 load, and PPU-facing output state.
- New call sites use views where they replace repeated `read_le_u16` / `write_le_u16` clusters cleanly.
- The views do not hide aliasing: shared scratch remains raw or explicitly named as shared scratch.

Verification:

- Byte parity still passes for the touched routes.
- Semantic snapshot diff remains empty on the same routes.
- RAM naming docs stay useful but become less central to normal code reading.

### Phase 4: Subsystem Graduation

A subsystem can move away from byte-layout storage only when it has:

- Semantic snapshot coverage.
- Route coverage that executes its normal and edge paths.
- Output coverage for PPU/OAM/audio side effects.
- A checkpoint bridge that can materialize legacy WRAM if needed.
- A rollback plan if semantic parity misses behavior byte parity caught.

Recommended order:

1. `OverworldMap16Load`: small state cluster, already named, good oracle coverage.
2. `Camera`: compact fields, high diagnostic value.
3. `Player movement core`: high value but high risk; do after camera/Map16.
4. `Inventory/menu-visible state`: good semantic domain, moderate route needs.
5. `Sprites` and `Ancilla`: delay until slot summaries and route coverage are stronger.
6. Shared scratch: last, and only where call-site lifetime proves it is local.

### Phase 5: Byte Parity Retirement by Subsystem

Byte parity should be retired only per subsystem, never globally.

For a graduated subsystem:

- Compare semantic fields every frame.
- Compare output-facing effects every frame.
- Optionally exclude the subsystem's private typed storage from raw WRAM comparison.
- Keep raw comparison for all non-graduated memory.

## Safety Rules

- Every semantic field must have a source address or typed-field source documented in code.
- Every ignored byte must have a reason and a stronger semantic/output check.
- Shared scratch is not eligible for typed ownership until proven local.
- PPU-facing state is not proven by gameplay semantics.
- Render parity and audio parity stay separate from state parity.
- A semantic snapshot passing does not prove byte parity; byte parity passing does prove the semantic snapshot should match if extraction is correct.

## First Implementation Slice

The first slice should be deliberately small:

1. Add `SemanticSnapshot` to `zelda_cpu_infra.rs`.
2. Extract fields from the existing `Snapshot` RAM/PPU data.
3. Add a semantic comparison report with named fields.
4. Add tests that mutate known fields and assert named diffs.
5. Add a lockstep CLI flag that prints the final semantic snapshot summary.

This creates the bridge without weakening any current oracle gate.

## Verification Matrix

Minimum checks for each slice:

```bash
cargo fmt -p zelda3
cargo test -p zelda3 semantic
cargo check -p zelda3-bin
python3 scripts/oracle_windows.py --run --only file-select-enter-game
```

When touching CLI or route scripts:

```bash
python3 scripts/oracle_windows.py --check --check-checkpoints
```

When touching renderer/PPU-facing semantic fields:

```bash
python3 scripts/oracle_windows.py --run --render --only <affected-window>
```

## Completion Criteria

This migration is complete when:

- Semantic snapshots can compare all gameplay, output, and route-visible state needed by current oracle windows.
- At least one low-risk subsystem stores typed state internally while still passing semantic/output parity against the C oracle.
- The raw WRAM comparison can be narrowed for that subsystem without reducing behavior proof.
- Docs explain exactly which subsystems still require byte parity and why.
