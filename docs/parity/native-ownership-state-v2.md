# Native ownership pilot: state v2

Step 2a, with the optional step 2b export change. Step 2c is deliberately pending
reviewer full-route v2 baselines. No native ownership ranges are enabled.

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
