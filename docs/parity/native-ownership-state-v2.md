# Native ownership pilot: state v2

Step 2a, with the optional step 2b export change, landed in `6fe0b9fb`.
The original verification below predates production ownership ranges. Current
step 2c results follow at the end of this document.

## Hash contract and audit

`canonical_state_hash` streams FNV-1a-64 over `zelda3-state-v2`, `wram_image()`,
SRAM, the existing non-GameState bincode tuple, then explicit per-sub-state
`hash_non_wram_fields` methods. See [the wire contract](state-ledger.md) for the
exact order and the grouped WRAM audit.

Every included non-WRAM tuple value is retained from v1:

| Value | Why it is included |
|---|---|
| `initialized` | Native initialization lifecycle, not a cartridge byte. |
| `apply_links_movement_to_camera_called` | Native per-update call guard. |
| `wanted_zelda_features` | Requested engine feature configuration. |
| `previous_host_controller_input` | Host input history used across updates. |
| `dialogue_blk_index as u64` | Native compressed dialogue block cursor. |
| `dialogue_font_blk_index as u64` | Native font block cursor. |
| `dialogue_flags` | Native dialogue decoder flags. |
| `dialogue_scroll_continuation` | Suspended native dialogue scrolling work. |
| `dialogue_fast_forward_hold_pending` | Native pending dialogue hold phase. |
| `dialogue_fast_forward_hold_active` | Native active dialogue hold phase. |
| `lag_free` | Engine execution mode. |
| retired thread-credit slot, `0u8` | Existing fixed wire slot; no live owner remains. |
| `gloves_color` | Native cached equipment palette selection. |

**Additional non-WRAM execution fields inside `GameState`: none found.** The
142 explicit state classifications follow their cartridge imports and publication
codecs, recursively through composite fields. A field is still WRAM-backed when
it writes only through a targeted setter, is mode-gated, or shares a cartridge
address with another view. In particular, resource/magic/progress fields, OAM
priority/allocator words, display cursors, and cached sprite banks are excluded
from the native tail because the image already represents them. Physical-address
work vectors' unused gaps and derived vector lengths are storage details.

`DisplayState.palette_provenance` is explicitly classified as **metadata**, not
execution state: its definition says it is derived metadata, excludes it from
snapshots, and ignores it for coherence. Its mirror words and annotation tags
would fingerprint cache/annotation history. The v1 exclusion is preserved.

`non_wram_hash!` expands a separate method per classified state. Exhaustive
patterns list every immediate field without `..`; adding a field requires an
explicit classification. Equipment uses the original field macro so all of its
expanded fields participate. No whole native struct is serialized into v2.

Record schema and sidecar metadata are now 3; metadata also carries
`state_domain: "zelda3-state-v2"`. Both direct Rust comparison and the Python
wrapper reject v1 baselines with a clear re-record message. Checkpoint layouts
are unchanged. Regression tests cover refreshing the entire native model from
the same RAM and the resource-adoption-after-save-clear case that changed v1.

## Narrow export/import

Each `NativeOwnedWramRange` has `export_owned(&ZeldaState, &mut [u8])` and
`import_owned(&mut ZeldaState, &[u8])` callbacks. These are narrow codecs, not
legacy `write_to_ram` projections: they must touch only their registered bytes.
`wram_image_with_registry` now makes one RAM copy and exports directly into it;
there is no scratch clone/reset for every range. An empty registry still returns
a borrowed image. Unowned import bytes copy to backing RAM; registered bytes
are adopted by the owner's codec.

The codec contract test runs against every production range as owners are added,
plus a nonempty test owner now. It tests export-after-import identity with three
input patterns and checks every foreign byte against three sentinel fills. The
existing poison/borrow/deferred-mirror tests remain. No production owner is
claimed by the test fixture.

## Step 2c plan

Once resources are the sole owner, remove their deferred mirror-adoption model.
For `CopySaveToWRAM`, replacement, and clear operations, dispatch each retired
cartridge byte/word to the actual native owner at the instant the ROM transfer
writes it. A word spanning two resource fields updates both bytes of their
native encodings at that boundary; unowned/reserved bytes still write backing
RAM. Whole-save helpers use that same transfer boundary and source order, rather
than clearing raw owned bytes and importing an entire struct later.

Preserve the word-at-a-time save/checksum observation and any intervening NMI:
exporting during the transfer must show exactly the writes already retired.
Subsequent source-ordered post-copy mutations then act on the current native
resources directly. Clear follows the same rule, so native bombs/magic change
when their zero stores retire. Keep the registry narrow and prove poisoned
backing bytes cannot affect reads, exports or v2 records. Use
`ZELDA3_ASSERT_NATIVE_COHERENT` for coherence diagnostics; do not restore the
removed one-off audit hook.

## Verification, 2026-10-23

Pinned executable: `target/state-ledger/zelda3-hash-v2`, SHA-256
`5c6f61a47e27dcbfde4358f4949a351a60ea101cceb02f9b88fc5dc5b0ba2618`.
Source base: `44d2d5e66990baf0461d05b61bd27f7c24758282`, branch `codex/native-truth`,
with this uncommitted change. Artifacts: `/tmp/zelda3-state-v2.K9eVGG`.

- `cargo build --profile parity -p zelda3-bin`: passed, no warnings.
- `cargo test --profile parity -p zelda3 --lib --no-run`: passed, exactly 24
  existing warnings; no new warnings.
- `CARGO_TARGET_DIR=target/release-check cargo check --release -p zelda3-bin --no-default-features`:
  passed, no warnings.
- `ZELDA3_ASSETS=$PWD/zelda3_assets.dat cargo test --profile parity -p zelda3 -p snes --lib`:
  SNES 504 passed / 7 ignored; Zelda 2,450 passed / 3 ignored / one known failure:
  `cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
  The expected mismatch remains `Some(1047868)` versus `Some(170442)` at
  `cycle_models/spotlight_caller_return.rs:410`. All six WRAM pilot tests passed.
- `cargo test --profile parity -p zelda3-bin state_ledger::tests`: all six passed.
- Python syntax and `git diff --check`: passed.
- Retained schema-2/v1 baselines were rejected both through sidecar validation
  and by the direct binary without sidecar validation, with the re-record message.

Record commands, all using the pinned executable and fresh output paths:

```sh
for mode in default lag-free; do
  for repeat in a b; do
    python3 scripts/state_ledger.py record \
      "/tmp/zelda3-state-v2.K9eVGG/${mode}-${repeat}-200000.jsonl.zst" \
      --frames 200000 --mode "$mode" \
      --binary target/state-ledger/zelda3-hash-v2
  done
done
```

| Mode | Run | Frames | Wrapper seconds | Frames/s |
|---|---|---:|---:|---:|
| default | a | 200,000 | 333.912 | 599.0 |
| default | b | 200,000 | 282.500 | 708.0 |
| lag-free | a | 200,000 | 198.144 | 1009.4 |
| lag-free | b | 200,000 | 189.863 | 1053.4 |

Early recordings overlapped the exhaustive library tests. Each pair is
byte-identical as compressed files and as decompressed JSONL. Every file has
exactly 200,000 contiguous schema-3 records (frames 0–199999), the correct mode,
and state/display/SRAM/audio fields. The modes have different contents.

Default compressed SHA-256:
`18b58a3f887457d770918497218eaf7eb39360e1d04ed789c8b04b0ff4d8ba77`.

Default JSONL SHA-256:
`fdac68ceb16dc43fdd4ffe096e86c534df6d6bfcbed1ec58930986f2b68d113b`.

Lag-free compressed SHA-256:
`469e85e833da6d734fa09cf7060df9e9dd63ea1030d3c65f804d11f5904bcf01`.

Lag-free JSONL SHA-256:
`628f839857b1794cb459b86d5fd5c38f53e8c0d40c7cc787ba8297418a12463d`.

Both modes also match the first 200,000 retained v1 baseline records in every
field except `schema` and the intentionally redesigned `state` hash. This is a
prefix behavior-regression check, not a full-route result or an external A/V
parity claim. `determinism-proof.json`, per-mode byte/prefix proof files, all
recording sidecars and logs, and build/test logs are retained with the artifacts.

`ZELDA3_POISON_OWNED_WRAM=1` comparison also matched every record and EOF
for **200,000 frames in each mode**, with the registry still empty. Default
comparison took 185.641 seconds; lag-free took 206.952 seconds. Commands:

```sh
for mode in default lag-free; do
  ZELDA3_POISON_OWNED_WRAM=1 python3 scripts/state_ledger.py compare \
    "/tmp/zelda3-state-v2.K9eVGG/${mode}-a-200000.jsonl.zst" \
    --frames 200000 --mode "$mode" \
    --binary target/state-ledger/zelda3-hash-v2
done
```

No GPU or Snes9x execution, oracle-cache writes, commits, pushes, `cargo fmt`,
engine field migrations, or repository one-off scripts were used. The reviewer
must record full-route v2 baselines before step 2c proceeds.

## Step 2c: resources and equipment

The first owner now stores its live bytes only in native fields:

| Owner | Inclusive ranges | Bytes |
|---|---|---:|
| PlayerResourcesState, including magic | `004ca`, `0f343`, `0f34f`, `0f360..0f377`, `0f379..0f37b` | 30 |
| InventoryItemsState | `0f340..0f342`, `0f344..0f34e`, `0f350..0f35f` | 30 |

The registry combines these disjoint fields into three contiguous ranges:
`004ca`, `0f340..0f377`, and `0f379..0f37b`. Its narrow codecs read/write
individual owner bytes, including either half of native words. Equipment,
bottles, and the reserved equipment byte are lossless. Bombs and bottle selection
remain in resources. Neither owner has a competing projection in the ownership
audit.

Migration counts: 44 production resource mutators, 10 production equipment
mutators, five magic mutators, and eight test-only setters now change the owner
without publishing RAM. Thirteen raw address-reading sites were redirected:
five menu inventory scans, the wishing-pond lookup, two bunny capability checks,
two dungeon-return rupee reads, and three ItemBus/HDMA reads. The existing
semantic HUD, sprite, and ancilla consumers continue reading these same owners.

Deleted synchronization includes 46 resource `sync()` calls and their coherence
assertions, four equipment bulk publications and the award-byte publication,
five magic publications, both master projections, and six deferred import calls.
Magic's import/publication helpers and equipment capability re-adoption are gone.
External image seeding still uses the registry codecs. Legacy refreshes preserve
the live resource/equipment structs.

Save clear, replacement, and extended room-word aliases retire each source word
through the byte owners. A mixed equipment/resource word is visible immediately;
there is no later resource adoption. Checksum portions still export at their
observation points. Startup cycle instructions use temporary wire images and
retire them before native callbacks; they never materialize owned values into
backing RAM. Cold `$55` fill, zero resets, replay patches, and both provisional
overworld rollback paths also respect the ownership boundary.

`PLAY_CRASH_CHECKPOINT_MAGIC` is `Z3RSPC23`. Field layout is unchanged, but old
paired checkpoints may contain native resource fields saved before deferred
adoption. They must be rejected rather than treated as authoritative owners.
The old-header rejection test passes.

### Resource-owner verification

Artifacts: `/tmp/zelda3-ownership-2c.zP2HU7`. Baselines were recorded for
200,000 frames per mode using the unchanged `target/state-ledger/zelda3-native2`
executable, SHA-256 `5c6f61a47e27dcbfde4358f4949a351a60ea101cceb02f9b88fc5dc5b0ba2618`.
Their compressed SHA-256 values remain the default and lag-free values listed
in the original v2 verification above.

Final owner executable: `zelda3-resources23` in that artifact directory,
SHA-256 `a377750b763b603b5e4e68b7a23e35c2d79d58622134307560a1d5393c37363c`.

| Mode | Normal comparison | Poison comparison |
|---|---|---|
| default | 200,000 records + EOF matched, 244.745 s | 200,000 records + EOF matched, 308.426 s |
| lag-free | 200,000 records + EOF matched, 240.654 s | 200,000 records + EOF matched, 308.224 s |

The 20k lag-free coherence smoke reports zero new `(frame, step, domain)`
entries: 440,709 before versus 440,700 after, removing nine inventory entries.
See `coherence-owner23-comparison.json` and the paired logs.

- Parity binary build: passed without warnings.
- Parity Zelda library test compilation: passed with the same 24 warnings.
- Release check without default features: passed without warnings.
- Full SNES/Zelda library suite: SNES 504 passed / 7 ignored; Zelda 2,450
  passed / 3 ignored / only the known spotlight-caller-return failure. It is the
  unchanged line-410 `Some(1047868)` versus `Some(170442)` mismatch. The Zelda
  run completed in 902.96 seconds; see `tests-resources-final-gate.log`.
- Dual-ownership audit: 114 to 112 projecting structs, 86 to 84 master-reachable
  structs, zero to three registered ranges / 60 bytes. Existing 60 overlap bytes,
  nine informational bridge overlaps, and zero high-risk same-mode overlaps are
  unchanged; the script retains its existing exit 1.
- `git diff --check`: passed.

Updated tests seed owned bytes through the registry or owner setters, compare
complete cartridge images with `wram_image()`, and use owner-aware byte reads
inside scalar assertion loops. This includes native inventory/player/magic
tests; item-award, player-component, player-collision, tile-behavior, and
save-progress frozen receipts; HUD/refill and dungeon-return tests; the item and
spotlight bus fixtures; and source-cycle/runtime snapshot comparisons. Frozen
golden files were not changed. The save-alias test now requires immediate
resource retirement instead of the obsolete deferred mirror expectation.

These are bounded Rust regression checks, not a full-route or external A/V
claim. No GPU, external Snes9x execution, oracle-cache writes, commits, pushes,
`cargo fmt`, or repository one-off scripts were used.

## Step 2c: save-progress owner

SaveProgressState adds 687 exclusive bytes, with these inclusive ranges:
`00202..00204`, `0040c`, `00656..00658`, `0f000..0f27f`,
`0f3c5..0f3ca`, `0f3e7..0f406`, and `0f4fe..0f4ff`. The complete registry
now has ten ranges and 747 bytes. This includes 320 room words, fourteen palace
death words, the two death counters, checksum, six story bytes, and seven HUD/
runtime bytes. The byte codec preserves the untouched half of every word,
including the odd-address death records and words split between domains.

All 25 bridge mutators now write only the owner. The four byte/word/HUD/room
publication helpers, runtime master projection, seven-byte constructor import,
live-save re-adoption, and save-progress RAM coherence assertions are removed.
The shared live word reader uses the same owner-aware byte lookup as ItemBus.
Room aliases, clear/copy words, checkpoints, and source-cycle accesses therefore
observe the current owner at the access point. Unregistered save bytes still use
backing RAM.

The final boundary audit also redirects eleven generic store sites (two properties,
one exit bus, four overlay, four spotlight) and both exit-bus WRAM read sites
through the byte gateway. In particular, the properties caller's `$040c` word
previously published the owned low byte both before and after its semantic store.
A regression now checks six owned byte destinations, unchanged poisoned backing
storage, and immediate visibility of the unowned `$040d` companion byte. This
candidate supersedes the initial progress24 binary and repeats its full matrix.

The former out-of-range HUD-slot implementation briefly held a clamped R mirror
while publishing the primary selection into the R wire byte. The sole owner now
holds that published value immediately. Frozen save receipts remain unchanged.
`PLAY_CRASH_CHECKPOINT_MAGIC` is `Z3RSPC24`: older checkpoints can retain deferred
progress-runtime mirrors even though the positional field layout is unchanged.

Verification executable: `/tmp/zelda3-ownership-2c.zP2HU7/zelda3-progress24-boundary`,
SHA-256 `3e8092de3181a7d3325eafbd38caa6d77533958feb1abf3ded80242aebc540f8`.
The original pinned native2 baselines are reused without re-blessing.

| Mode | Normal comparison | Poison comparison |
|---|---|---|
| default | 200,000 records + EOF matched, 200.958 s | 200,000 records + EOF matched, 197.152 s |
| lag-free | 200,000 records + EOF matched, 207.834 s | 200,000 records + EOF matched, 193.363 s |

- The final 20k lag-free coherence smoke has zero new reports and the same
  440,700 `(frame, step, domain)` entries as the accepted resource-owner run.
  See `coherence-progress-boundary-comparison.json` and the paired logs.
- The final parity binary build passes without warnings. Library test compilation
  retains exactly the same 24 warning headings. The final release check without
  default features passes without warnings (20.42 seconds).
- Full library acceptance: SNES 504 passed / 7 ignored; Zelda 2,451 passed /
  3 ignored / only the known spotlight-caller-return failure (1,005.78 seconds).
  The failure is the unchanged line-410 `Some(1047868)` versus `Some(170442)`.
  See `tests-progress-boundary.log`. Repaired fixtures cover sprite-prefix
  handoff, staged overworld screen comparison, rain instruction/resume edges,
  witch room-word flags, smithy progress flags, and room-tag handoff. The latter
  now compares both exported images, including owned `$04ca`. All six repairs
  and the new properties no-publication regression pass in this full run.
- The ownership audit moves 112 to 111 projecting structs and 84 to 83 master
  projections. Its existing 60 overlap bytes, nine informational bridge overlaps,
  zero high-risk overlaps, and exit 1 remain unchanged.

Fixture updates preserve the original source inputs: seed generators write an
external cartridge image and import it through the registry. Custom fixture
buses use owner-aware reads and stores. Complete comparisons use exported images;
individual byte assertions use the fast byte accessor. The native progress
bridge tests now require backing RAM to stay unchanged. Historical standalone
projection hashes explicitly export the seven runtime bytes into their test
buffers. No frozen golden data was changed.

## Step 2c stop: frame ownership needs explicit mode context

Stopped at FrameState, before changing its storage or registering any of its six
bytes: `00010..00011`, `0001a`, `000b0`, `0010c`, and `00fc1`.
SystemSignalsState is also unchanged; it was not advanced past this stopping point.

This is a cross-owner API dependency, rather than an overlapping FrameState
projection reported by the ownership scanner:

- `SaveLoadTransferState::write_to_ram` (`native/misc.rs:348`) reads
  `RamTarget::read_byte(MAIN_MODULE)` to decide whether `$0000` is save-transfer
  state or active dungeon sprite/OAM scratch. The master currently publishes
  FrameState first (`native.rs:483`), implicitly supplying this mode context.
- `NativeDungeonMapDisplayBridgeMut::sync_marker_offsets` (`native/misc.rs:625`)
  reads backing `ram[$0010]` before publishing `$0fa8/$0faa`. In module 7 these
  bytes are live dungeon hitbox work, not the map's stale marker offsets.
  The adopting bridge holds only the map state and `&mut [u8]`, with no
  FrameState reference or explicit mode parameter.
- Three further mode gates in coherence reporting (`native.rs:450`,
  `native/misc.rs:406`, and `native/world.rs:2203`) also consume the same byte.
  These gates must receive the actual owner context when migrated.

With native `main_module = 7` and poisoned backing `$0010 = $a5`, the current
save-transfer projector takes its non-dungeon branch and writes `$0000`; the map
bridge likewise takes its marker-publication branch and overwrites hitbox work.
This follows directly from the two guards above. Removing frame publication and
adding a narrow frame codec cannot repair those consumers: both APIs lack access
to the frame owner. Re-exporting frame bytes into RAM would violate sole storage.
The prerequisite is to thread explicit native mode context through these
projector/bridge APIs and their callers, then migrate every frame reader/writer
and run a separate ownership matrix. No partial frame conversion was retained.

Audit evidence: `whoowns-frame-main.txt` and `dual-progress-boundary.txt` in the
same `/tmp/zelda3-ownership-2c.zP2HU7` directory. The accepted work remains ten
registered ranges / 747 bytes, checkpoint magic `Z3RSPC24`. Final
`git diff --check` passes. No commit or push was performed.

## Step 2d: FrameState

FrameState exclusively owns `00010..00011`, `0001a`, `000b0`, `0010c`, and
`00fc1`: six bytes in five narrow registry ranges, fifteen ranges / 753 bytes
overall. Its master projection, adopting bridge, eager publications,
re-adoption, and frame-versus-backing assertions are removed. Callers borrow
the owner directly; existing write-watchpoints live on its methods. Legacy
mirror refreshes preserve the live frame owner.

Save-transfer publication and dungeon-map marker synchronization receive the
module explicitly from ZeldaState. The three coherence mode gates also use
native context. Live mode decisions no longer consume backing `$0010/$0011`.
Imports, detached source machines, and retained display images still decode
their own cartridge-format images.

The byte/word gateway retires frame stores, including words spanning owned
and unowned bytes. Generic startup/NMI accesses and DMA sources use owner-aware
reads or exported images. Link graphics' separate DMA slice helper gathers
only the requested bytes. Tileset and sheet cycle models accept byte readers:
wrapped low-WRAM art and back-references sample the owner at the instruction,
without cloning WRAM per instruction. Source fixtures and zelda3-bin
diagnostics use exported images.

Display composition keeps the original backing buffer aside, supplies an
exported live image to the outgoing snapshot, and installs the captured frame
owner while composing that generation. It restores the original native state
and backing buffer afterward, preserving distinct captured/live mode contexts.

Checkpoint magic is `Z3RSPC25`. Positional fields are unchanged, but older
checkpoints can contain frame mirrors awaiting adoption and are rejected.
Fixtures seed and compare owners or exported images; frozen golden data is
unchanged. New regressions cover split stores with poisoned backing and both
mode-gated publishers with native module 7 versus poisoned backing `$0010`.

### Frame verification

Base: `6d75a672d0686dd8790369af75d68b7acfcbaf2d`, branch `codex/native-truth`,
with uncommitted changes. Artifacts: `/tmp/zelda3-ownership-2d`. Executable:
`zelda3-frame25-final`, SHA-256
`3f76e293b4def5c31db287effdd2194527af1294f1327c0f88658355c9bce25b`.
`frame25-source.patch` retains the source changes. The supplied native2 baselines
in `/tmp/zelda3-ownership-2c.zP2HU7` were reused without re-blessing; their
compressed hashes remain `18b58a3f...4d8ba77` and `469e85e8...0bcf01`.

| Mode | Normal, 200k records + EOF | Poison, 200k records + EOF |
|---|---|---|
| default | matched, 275.211 s | matched, 252.512 s |
| lag-free | matched, 238.111 s | matched, 198.145 s |

- The 20k lag-free coherence smoke matches the accepted 440,700-entry set
  exactly: zero new and zero removed `(frame, step, domain)` tuples. See
  `coherence-frame25-comparison.json` and `coherent-frame25.log`.
- Parity binary build passes without warnings. Final library test compilation
  passes with exactly 24 warnings (`lib-no-run-frame25-wire-fixtures.log`). Release
  check without default features passes without warnings (`release-final25.log`).
- Full library acceptance: SNES 504 passed / 7 ignored; Zelda 2,453 passed /
  3 ignored / only the known spotlight-caller-return failure (983.08 seconds).
  See `tests-frame25-accepted.log`. Frozen projection fixtures explicitly encode
  the six frame bytes; spotlight fixtures construct an external seed image
  before importing it. Their original inputs and frozen golden files are unchanged.
- Audit: 111 to 110 projecting structs, 83 to 82 master-reachable projections.
  Existing 60 overlap bytes, nine informational bridge overlaps, zero high-risk
  overlaps, and exit 1 are unchanged (`dual-frame25-final.txt`).
- `python3 scripts/loc.py HEAD` before and after reports 558,636 production Rust
  lines at the unchanged HEAD. The tracked working tree has 558,613 by the same
  nonblank-line/file classification: a reduction of 23. `git diff --check` passes.

These are bounded Rust regression checks. No GPU, external Snes9x execution,
oracle-cache writes, commits, pushes, `cargo fmt`, or repository one-off scripts
were used. FrameState is accepted; SystemSignalsState verification follows.


## Step 2d — SystemSignalsState sole ownership

SystemSignalsState owns 71 bytes: 19 scalar signal bytes and the 26 meaningful
bytes in each MSU resume record. Twelve additional registry ranges bring the
pilot to 27 disjoint ranges / 824 bytes. MSU trailing reserved bytes and the
space between records remain ordinary WRAM. Partial word stores replace only
the addressed little-endian byte, including mixed owned/unowned words.

The adopting bridge, master/scratch projections, and backing-coherence assertions
are removed. Callers mutate the owner directly; legacy refreshes preserve it.
Display composition installs the captured signals alongside the captured frame
owner, then restores the live native state and original backing buffer.
Replay watches and diagnostic dumps read through the gateway or exported images;
the unused compatibility view and its range-copy adapter are deleted.

The source-store audit also caught final backing copies after native stores in
the entrance, load-file parent, room-entry, and player drawing paths. Those
copies now retire bytes through the gateway, including the previously claimed
frame bytes. Graphics scratch clearing handles the frame pause byte through the
word gateway. The intro column-clear helper accepts a word-clear callback, so
its large WRAM clear also reaches the two native MSU records. Regression checks
cover poisoned MSU backing during that clear and the player drawing word crossing
into the game-over byte. These final boundary corrections are included in the
cumulative SystemSignalsState proof below; the earlier Frame-only executable
predates them.

Checkpoint magic is now `Z3RSPC26`; old checkpoints with pending signal mirrors
are rejected. Frozen fixture data is unchanged. Fixtures construct external
seed images before import and compare exported images or native fields.

### System verification

Final executable: `/tmp/zelda3-ownership-2d/zelda3-system26-final`, SHA-256
`40b8cdfa755b9481ea9abdcb5acff7d9781d062d7047e1acfd0a09f9b34ba9a0`.
Earlier `system26` and `system26-boundary` runs are superseded by this final
executable. The supplied 2c baselines are unchanged (compressed SHA-256 hashes
`18b58a3f887457d770918497218eaf7eb39360e1d04ed789c8b04b0ff4d8ba77` and
`469e85e833da6d734fa09cf7060df9e9dd63ea1030d3c65f804d11f5904bcf01`).

- The final 20k lag-free coherence smoke matches the accepted 440,700-entry set
  exactly: zero new and zero removed tuples (`coherence-system26-final-comparison.json`).
- Parity binary build passes without warnings (`build-system26-final.log`).
  Library test compilation passes with exactly 24 warnings
  (`lib-no-run-system26-final-fixtures.log`). Release check without default
  features passes without warnings (`release-system26-final.log`).
- Six focused fixture/ownership checks pass (`focused-system26-final-fixtures-current.log`).
- Audit: 110 to 109 projecting structs and 82 to 81 master-reachable projections.
  Existing 60 overlap bytes, nine informational bridge overlaps, zero high-risk
  overlaps, and exit 1 are unchanged (`dual-system26-final.txt`).
- `python3 scripts/loc.py HEAD` before and after reports 558,636 production Rust
  lines at unchanged HEAD. The tracked working tree has 558,655 by the same
  classification: System adds 42 after Frame's reduction of 23, a net increase
  of 19 from 2c. Inline test modules count as production under this script.

| Mode | Normal, 200k records + EOF | Poison, 200k records + EOF |
|---|---|---|
| default | matched, 226.399 s | matched, 280.070 s |
| lag-free | matched, 201.554 s | matched, 288.173 s |

See `compare-system26-final-*.log`.

Full library acceptance (`tests-system26-final-accepted.log`): SNES 504 passed /
7 ignored; Zelda 2,454 passed / 3 ignored / only the known
`spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`
failure (960.31 seconds). Its unchanged assertion at line 410 reports
`Some(1047868)` versus `Some(170442)`. The command was
`ZELDA3_ASSETS=$PWD/zelda3_assets.dat cargo test --profile parity -p zelda3 -p snes --lib`.
The final graphics fixture correction changes only its expected-image seed;
the pinned production executable is unchanged.

`native-ownership-2d-final.patch` retains the cumulative source/report diff in
the artifact directory. `git diff --check` passes. Branch and HEAD remain
`codex/native-truth` / `6d75a672d0686dd8790369af75d68b7acfcbaf2d`.
Both requested owners are complete under these bounded Rust regression gates.
No GPU, external Snes9x execution, oracle-cache writes, commits, pushes,
`cargo fmt`, or repository one-off scripts were used.
