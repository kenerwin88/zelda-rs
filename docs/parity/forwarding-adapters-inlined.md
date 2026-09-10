# Forwarding adapters inlined; the single-coordinate prep returns its record

The split of the sprite handlers into modules left behind adapters named
`<helper>_for_<module>` whose whole body was a call to the canonical helper
of the same name: eight of them, called from twenty-five sites, in the
sprite core, the dungeon NPC module and the Hinox shop module. The
dungeon NPC module also kept a three-axis move adapter equal to the
canonical `sprite_move_xyz`, and a prep adapter that declared a zeroed
coordinate record only to discard it. All are gone; the call sites name
the canonical helpers.

`sprite_prep_oam_coord`, the single-coordinate preparation, took a mutable
record like the multi-part draw helpers once did. It now returns the
record. Twenty-seven callers that read the record bind the return value;
twenty-one that called it only for its side effects (the sprite's pause
flag, the draw hitbox work, the off-screen despawn) call it and drop the
value; the Stalfos draw, which prepared in one branch and drew in the
other, binds the record only where it uses it. The two unit tests that
seeded the record's fourth byte to prove the preparation clears it now
check the returned record.

Compatibility constraints remain explicit:

- Every preparation call still happens at the same point, so its side
  effects on the sprite slot and the draw hitbox work are unchanged.
- The forwarders passed their arguments through unchanged, so the
  canonical helpers receive the same values in the same order.

## Verification

The preparation tests check the same coordinates, the cleared fourth byte
and the same slot effects as before. The ownership scanner is unchanged.
The library compiles with no warnings in the parity, dev, and lib-test
builds; readability and projection discovery pass. All 1,721 library tests
pass under the dev profile.
