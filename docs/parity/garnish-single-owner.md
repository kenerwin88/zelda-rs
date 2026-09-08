# Garnish slots: one live owner

Garnish slots now live only in `ZeldaState.ram`. `GarnishSlotView` and
`GarnishSlotMut` provide named access to those bytes without keeping a second
copy. The gameplay entry points remain `garnish_slot_view` and
`garnish_slot_view_mut`, so callers keep their existing domain operations.

## Storage and write contract

There are 30 slots and 13 byte fields per slot: type; X/Y low and high bytes;
X/Y velocity and subpixel bytes; countdown; associated sprite; floor; and
OAM flags. Each field is a 30-byte array. The owned regions are:

- `0x1f800..0x1f94a`: eleven arrays, through associated sprite.
- `0x1f968..0x1f986`: floor.
- `0x1f9fe..0x1fa1c`: OAM flags.

The gaps and adjacent banks belong to other data. A byte setter writes only
its selected address. Coordinate setters write the low byte followed by the
high byte in the separate coordinate arrays. Constructors reject slot 30
and higher, preserving the old slot-bank boundary.

Removed: `GarnishSlotState`, `GarnishSlotsState`, the `SpriteState` field,
load/projection/coherence plumbing, and the synchronized mutation bridge.
This removes 390 duplicated live bytes and the opportunity for a stale
native projection to restore them over current WRAM. `GarnishRuntimeState`
(active effect, collision scratch and other shared runtime fields) remains
a separate domain and is not part of this migration.

## Checkpoints and snapshots

`ZeldaState` already owns and serializes WRAM. A clone or restored checkpoint
therefore has its own garnish bytes; views borrow the corresponding state's
WRAM. Hardware latches and historical snapshots remain independent.

Removing the native bank changes positional bincode layout. Playable Rust
checkpoint magic advances from `Z3RSPC02` to `Z3RSPC03`; old Rust checkpoints
must be recreated and are rejected before decoding. The Snes9x oracle cache,
input route, SRAM format and cartridge data are unchanged.

## Verification

Baseline `3a220049` was replayed from zero for 200,000 frames with temporary
assertions comparing native garnish bytes against WRAM before every slot
read, mutation and bulk projection. All assertions passed, exact A/V matched,
both reached WRAM goldens matched, and endpoint WRAM matched the previously
validated build. The temporary assertions were removed with the duplicate bank.

The single-store candidate also matched all 200,000 A/V frames, both reached
WRAM goldens, and the baseline's complete 128 KiB WRAM endpoint byte for byte.

The contract tests freeze the original field addresses independently of the
production constants. They exercise all 390 bytes, byte and word boundaries,
neighbor preservation, raw-write visibility, stale-projection prevention,
and clone/bincode isolation. A checkpoint-loader test rejects the old layouts
before positional decoding. Full-route acceptance is recorded by the promoted
receipt linked from `routes/full_run/parity-frontier.json`.
