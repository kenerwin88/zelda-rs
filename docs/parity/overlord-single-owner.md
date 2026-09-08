# Overlord and boss scratch: one live owner

Overlord slots and boss home positions share `ZeldaState.ram`. Named
`OverlordSlotView`, `OverlordSlotMut`, and `ArmosKnightHomePositionMut`
access that storage directly. Gameplay callers use the existing named
`ZeldaState` helpers. `BossHomePositionRead` remains a captured value, not a
second live owner.

This is a synchronization cleanup, not the final gameplay architecture.
These views still depend on the SNES layout. Removing that dependency requires
semantic owners with explicit sharing and lifetimes, plus WRAM encoding at
compatibility boundaries after every relevant reader and writer is migrated.
Moving other native fields into WRAM is not an objective of this change.

The removed `OverlordSlotsState` stored 88 work bytes and eight spawned-area
bytes. `BossHomePositionsState` stored another 216 bytes in two overlapping
27-entry coordinate arrays. Removing both eliminates 312 duplicate data bytes,
26 synchronized overlord forwarding methods, their load/projection/coherence
plumbing, and the Arrghus reload that repaired stale overlord state after
writing puff homes.

## Shared address contract

The overlord work bank is `0x0b00..0x0b58`. Eight physical slots have separate
type, X/Y low/high, general-purpose, floor, and two-byte sprite-position
arrays. Boss routines intentionally index across those arrays: for example,
slot 8's X low byte is slot 0's X high byte. Constructors therefore do not
impose an eight-slot limit. Reads beyond the work bank retain the old zero
result; writes reject destinations outside it. Two-byte mutations validate
both destinations before publishing either byte, preserving rejection
without a partial WRAM write. The separate spawned-area array occupies only
`0x0cca..0x0cd2` and rejects index 8 and above.

Split X/Y coordinates, adjacent low-byte words, and circle coordinates have
different layouts. Their named operations preserve the original addresses,
low/high write order, wrapping arithmetic, and returned values.

Armos homes use bases `0x0b10`, `0x0b20`, `0x0b30`, and `0x0b40` for X low,
X high, Y low, and Y high. Arrghus reads use the same bases minus one.
The existing 27-entry home write surface is preserved, including its high-index
aliases into the sprite-position and sprite-stunned bytes. Invalid Armos
writes remain no-ops. The Armos reader still uses the overlord bank's zero
reads beyond its end: slots 24..26 read a zero Y high byte even though the
writer reaches that byte. Arrghus reads those high-index bytes directly.
These two home layouts represent 76 distinct bytes across `0x0b0f..0x0b5b`;
they must not become independent coordinate arrays.

## Snapshots and compatibility

WRAM is already owned, cloned, and serialized by `ZeldaState`; a view borrows
the particular state's storage. Hardware latches and historical snapshots
keep their own semantics. Other sprite and ancilla native banks remain
outside this migration.

File-selection state also reuses `0x0b10..0x0b17` and retains its existing
projection behavior. This migration removes the actor mirrors; it does not
claim every subsystem sharing these addresses has been migrated. Projection
tests specifically check that `SpriteState` cannot restore the removed bank.

Removing the fields changes positional bincode layout. Rust checkpoint magic
advances from `Z3RSPC03` to `Z3RSPC04`; old Rust checkpoints must be recreated
and are rejected before decoding. The immutable Snes9x oracle cache, input
route, SRAM format, and cartridge data are unchanged.

## Verification

Contract tests freeze addresses independently from production constants.
They cover all 26 overlord mutations and 16 getters, extended indices,
boundary rejection, raw-write visibility, stale-projection prevention, and
clone/bincode isolation. Boss-home tests cover all 27 positions, shared
aliases, captured values, invalid writes, and unrelated-byte preservation.
The checkpoint loader rejects obsolete layout headers before deserialization.

The ownership scan removes one reachable bulk writer (85 to 84); its existing
34 overlapping-byte findings are unchanged. Full-route acceptance is bound
to the promoted receipt in `routes/full_run/parity-frontier.json`.

Baseline `1f4936c8` replayed from zero for 200,000 frames with temporary
assertions comparing the overlord mirror against WRAM before every overlord
read, mutation, and actor-bank bulk projection. All assertions passed; exact
A/V and both reached WRAM goldens matched. The complete 128 KiB endpoint
also matched the preceding validated build. The probes were removed along
with the duplicate bank.

The candidate also matched all 200,000 audio/video frames, both reached WRAM
goldens, and the baseline's full 128 KiB endpoint byte for byte. All 1,736
library tests passed (two existing tests remain ignored), together with the
checkpoint rejection test and RAM readability guard. Production Rust is
715 lines smaller; the new contract tests are counted separately.

Candidate `4a6b2b6e` passed the full from-zero cached Snes9x route in
2486.55 seconds: all 1,581,079 video/audio frames matched, with no paired
resume and no reported RNG drift. All four WRAM goldens and the complete
final 128 KiB WRAM endpoint matched the preceding validated build. The
promoted receipt binds binary SHA-256
`f3cf96f94c546d7ec7fee06b29ec72e7800f58341a46461d95953558c84d7aa2`.

The normal commit hook also passed its standalone smoke and a fresh
180-frame live Snes9x comparison. The full-route evidence above comes from
the immutable cached oracle; it does not claim a fresh full-route core run.
