# Native player components

Equipment has one inventory owner. `FollowerLinkState` no longer stores or
projects Flippers or Moon Pearl. Swimming and bunny checks read inventory,
and bunny recaching receives the capability explicitly.

`PlayerMagicState`, inside player resources, owns amount, refill, and
consumption level. Spending, refunds, byte wrapping, and refill arithmetic
live in this native component. Resource publication deliberately excludes
amount; amount mutations publish just that byte. Full resource projection
includes all three fields. The heart/refill word still combines the heart
counter with the magic refill byte, preserving carry across the two domains.

The remaining 175 player fields are composed into four native components:

| Component | Fields | Responsibility |
| --- | ---: | --- |
| Movement | 88 | Position, velocity, floor, collision, swimming, and locomotion |
| Actions | 50 | Combat, transformations, item use, and action timers |
| Presentation | 29 | Animation, OAM, graphics staging, and draw state |
| Input | 8 | Filtered input and input history |

613 native methods move with their component. Cross-component operations
remain in `FollowerLinkState`, including the word spanning input's B-button
counter and the spin-attack delay in actions. Component fields are private
to the player boundary. Existing read access uses a small forwarding macro;
376 mutation/helper forwarding methods are unnecessary because the bridge
calls the owning component directly.

`player/compatibility.rs` contains the cartridge codec and player bridge.
The original full projection order and partial mutation writes remain
explicit. Native movement and action methods perform no RAM publication.
The original player module falls from 8,279 to 2,182 lines; this is an
ownership and organization change, not a claim of net code-size reduction.

## Observation boundaries

Legacy player entry points can follow raw save transfers and overlapping
save-word writes. They still observe the player state, equipment capabilities,
and magic amount at the established entry boundary, now into their respective
owners. Cost checks import consumption level before reading its native field.
The old independent player equipment copies and split magic fields are gone.
The remaining player reload is intentionally retained until its raw writers
and mode aliases can migrate component by component.

Rust checkpoint layout advances from `Z3RSPC06` to `Z3RSPC07`; layouts 01–06
are rejected before positional decoding. Cartridge saves and the Snes9x
oracle cache retain their formats.

## Validation

Baseline `0d2e692f` matched 200,000 exact cached audio/video frames, both reached
WRAM goldens, and used binary SHA-256
`836c301339977984043f0aa698f411152c3645d71fb618e719a2b861fae4d31e`.
The ownership-only batch also matched all 200,000 frames, both goldens, and
the entire 131,072-byte baseline WRAM endpoint.

`player-components-0d2e692f.txt` freezes 32 old-implementation cases before
the refactor. Each hashes full WRAM after projection, cross-component
mutations, and subsequent projection. It has no regeneration switch.
Additional tests exhaust every byte-valued magic amount/cost pair, cover
independent snapshots and scoped publication, and exercise borrowing across
the input/action word. All 1,752 library tests pass, with two existing ignored
tests. The checkpoint rejection test and RAM readability guard also pass.

The ownership tools now include nested native modules, and the projection
scanner follows publication helpers. A regression checks codecs in separate
files without attributing child projections to both parent and child. Running
the improved scanner against both baseline and candidate reports the same
40 potentially overlapping bytes; its broader helper coverage exposes six
pre-existing bytes that the old scanner missed. Static overlap reports are
not evidence of a runtime conflict. The moved native/bridge methods were also
compared against the baseline after normalizing component paths and formatting;
no unintended method-body changes were found.

The final component candidate matched all 200,000 consecutive cached A/V
frames in 296.74 seconds, both reached WRAM goldens, and the complete baseline
WRAM endpoint. A subsequent visibility-only change restricts native magic
mutations to the native-state module; gameplay continues through its publishing
bridge. The full cold gate will validate the committed source and binary.
