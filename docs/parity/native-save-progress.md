# Native save progress

`SaveProgressState` owns 320 dungeon-room records, six story/world progress
bytes, fourteen palace death counts, pending/total death counts, and the
save checksum. Its existing six runtime/HUD bytes remain separate fields.
The 1,280-byte `dungeon_info` cache is gone. Equipment, resources, followers,
overworld events, and reserved save bytes no longer have copies in this state.

The room records end at `0xf280`, where overworld event storage begins.
The source room index can extend into the rest of the save block. The
`saved_room_flags` family routes ordinary rooms to native records and extended
indices through the cartridge codec, preserving the original 640-word bound.
Those cross-domain writes retain the other owners' import timing and refresh
progress at that explicit transfer boundary. This keeps the alias contract
without enlarging the room array to include foreign inventory/event bytes.
An exhaustive 641-index regression covers every word and the first rejected
index, including words that straddle the named progress fields.
Progress owns `0xf3c5..0xf3cb`, `0xf3e7..0xf407`, and the checksum at
`0xf4fe..0xf500`. Gameplay reads and mutations use the named fields/records.
Dungeon loading and the credits death display now read this native owner.
The credits table explicitly selects palace counts or total deaths: the old
table's final index 15 addressed the total at `0xf405`, beyond the fourteen
palace entries. A frozen-table regression preserves that intentional alias.
The serialized progress state occupies 686 bytes instead of 1,294, removing
608 serialized bytes and the old save-bank heap allocation.

## Compatibility and timing

`game_state/save_format.rs` defines the original cartridge layout. `LiveSave`
borrows the published WRAM image without caching another live bank. It copies
both SRAM mirrors before checksum calculation, preserving all reserved bytes.
The checksum reads a caller-selected prefix and resumes against the current
image for the remaining words. It must not use the earlier SRAM copy or a
single snapshot across an NMI boundary. The original save format is unchanged.

`clear_live_save` and `replace_live_save` are explicit whole-save boundaries.
They import the progress fields after transferring the compatibility bytes.
Other native owners retain their established import/frame schedule and the
source-ordered writes following `CopySaveToWRAM`; eagerly refreshing all of
them here would be a separate behavioral change. Ordinary progress mutations
publish only their selected byte or word. The frame projection retains its
existing six runtime/HUD writes.

The bridge no longer reloads saved progress before mutations. It still imports
the six runtime/HUD bytes, which have a legacy compatibility contract: an
out-of-range HUD slot clamps the native write to R while its published value
uses the first-slot fallback. The next bridge construction historically
observes that published value. The frozen regression includes this case.
Removing that separate runtime surface requires migrating its consumers.

The playable Rust checkpoint header advances from `Z3RSPC05` to `Z3RSPC06`
because the positional native-state layout changed. Old Rust checkpoints are
rejected before decoding; the Snes9x cache and cartridge saves remain usable.

## Validation

Baseline `38a9515b` passed the 180-frame live Snes9x gate and the from-zero
200,000-frame cached A/V comparison. Both reached WRAM goldens matched.
The baseline binary SHA-256 is
`9a8b7dc380eb302e07e51750f0541656f9963e3063ef680602512338f797b41d`.

`save-progress-effects-38a9515b.txt` freezes 32 old-implementation cases.
Each case updates all 320 room records, interleaves equipment/resource and
progress mutations, and hashes full WRAM after mutation, projection, import,
HUD refresh, clear, and post-clear updates. It was captured before replacing
the native model and has no committed regeneration switch. It is a Rust
regression fixture; Snes9x A/V remains the route authority.

Additional contracts cover all 640 checksum-prefix boundaries with changes
to both earlier and later bytes between slices, all three save slots, primary
and backup bounds, reserved-byte preservation, exact progress ownership,
native mutations with a deliberately different compatibility image, and
independent clone/bincode snapshots.

The checkpoint rejection test covers layouts 01 through 05. The ownership scan
remains at 111 writers, 84 reachable writers, and 34 existing overlapping bytes,
with no new overlap.

During implementation, a bounded-only room adapter passed 200,000 exact A/V
frames and both reached goldens but differed in the final WRAM byte `0xf304`.
The original extended room index writes that event byte. The general save-word
adapter restores this behavior; no frame- or room-specific exception is used.

The completed candidate passed all 1,747 library tests (two existing ignored
tests) and the RAM readability guard. Its from-zero 200,000-frame cached A/V
run matched every audio/video frame in 300.81 seconds, both reached WRAM
goldens, and all 131,072 bytes of the baseline's WRAM endpoint.

Source commit `b49d1c55` passed the full from-zero cached Snes9x gate in
2418.03 seconds: all 1,581,079 consecutive audio/video frames matched, with
no checkpoint resume and no reported RNG drift. All four WRAM goldens and
the complete final 128 KiB WRAM endpoint matched the preceding promoted
native-item-awards build. Binary SHA-256:
`836c301339977984043f0aa698f411152c3645d71fb618e719a2b861fae4d31e`.

The source commit's normal hook passed the standalone smoke and a fresh
180-frame live Snes9x comparison. The full-route proof uses the immutable
cached Snes9x oracle; it does not claim a fresh full-route core execution.
The promoted manifest is committed in
`routes/full_run/receipts/native-save-full-av.manifest.json`.
