# Lag-free logical parity

The engine's lag-free mode (`ZeldaState::set_lag_free(true)`) runs every frame
as one complete main-loop iteration: work the cartridge spreads over lag frames
finishes inside the iteration. Host-frame parity against Snes9x cannot judge
such an engine, because a lag frame on the cartridge is simply a slow
iteration. Lag-free parity compares **per main-loop iteration** instead.

## The two tools

The corrected full-route ledger is
`/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2`.
`full` remains the immutable backup. The recorder fix, exact ledger comparison,
continuous pass and both seeded census passes are documented in
[Continuous lagless 2](cost-pacing.md#continuous-lagless-2). Use a fresh output
directory when recording another ledger.

```sh
# 1. ROM ledger: the cartridge on the in-repo source CPU, cut into iterations.
cargo build --profile parity -p snes --example logical_frame_ledger
C=.git/parity-oracle-cache/<key>
ZELDA3_LEDGER_SEED_EVERY=2000 ZELDA3_SOURCE_PROGRESS_EVERY=100000 \
  target/parity/examples/logical_frame_ledger saves/zelda3.sfc \
  $C/initial.srm $C/input.txt 1581079 <ledger-dir>

# 2. Lag-free replay of the engine against it.
target/parity/zelda3 --logical-replay <ledger-dir> $C/initial.srm \
  --seed-every 2000 --video $C --report 0 > census.txt

# 3. Worklist: totals and first divergences grouped by module and variable.
scripts/logical_census_summary.py census.txt
```

`--start K` seeds at iteration K and skips everything before it (a segment
repro in seconds); `--dump-wram K:DIR` writes both sides' WRAM after
iteration K; `--apu-trace lo-hi` prints both sides' APU port activity.

The ledger (`crates/snes/examples/logical_frame_ledger.rs`) runs the cartridge
from reset on `Snes9xColdCpuExecutor`. An iteration ends at an NMI accepted
while `$12` is clear (Main finished; the handler then performs the frame's
updates); an NMI with `$12` set is a lag NMI inside the iteration. Testing the
interrupted PC against the `$00:8034` wait loop instead merges iterations: Main
can finish and be interrupted before it reaches the loop. The handler ends at
its `RTI` (IRQs can nest). A source receipt can also accept a new interrupt
after that RTI; the recorder retires the return before accounting for the new
interrupt, so the post image cannot slip to the next NMI. Per iteration it
records, in `iterations.jsonl`:

- the joypad word the opening NMI latched (`$4218`, the bit-reversed libretro
  word);
- the cartridge RNG outputs (the `STA $0FA1` at `$0D:BA7C`, with carry);
- APU port writes and reads, lag NMI and IRQ counts;
- `picture_host`: the host whose scanned-out frame first shows the
  iteration's work (cache frame N is `retro_run` N).

It also writes XOR-delta WRAM streams (images at each closing NMI's acceptance
and return) and, with `ZELDA3_LEDGER_SEED_EVERY`, full machine seeds
(WRAM, VRAM, CGRAM, OAM, SRAM, `PpuState`) to restart the engine from.

The replay (`zelda3-bin/src/logical_replay.rs`) runs a lag-free romless
engine and checks after every iteration as below. At each seed boundary an
engine that drifted (any WRAM, RNG, APU command or video content miss in the
last 200 iterations, or a panic) is replaced by a fresh one seeded from the
ledger; an engine still in step continues, because a seed into the middle of
some sequences cannot rebuild every runtime owner (`--eager-seed` reseeds at
every boundary). Seeding moves the cartridge's live HDMA table (`$1B00`) into
the port's dynamic table.

| Check | Against | Notes |
|---|---|---|
| WRAM | post-NMI ROM image | `PORT_REPRESENTATION` lists bytes the port legitimately stores differently (scratch, stack, own flags, ROM pointers, relocated HDMA table) |
| Video | Snes9x cache frame `picture_host` | misses classed `early`/`late` (one-iteration shift), `content_lag` (the ROM lagged: HDMA tearing a lag-free frame never shows), `content_tear` (the closing NMI handler returned inside active display, recorded as `nmi_return_line`, so its scroll and register writes apply partway down that frame), `content` |
| APU commands | ledger port writes | port transitions to nonzero values, matched within one iteration; commands a lag NMI sent and Main then replaced are `unmatched_lag` |
| RNG | ledger outputs | draws past an iteration's outputs (`extra`) or outputs left unused (`unconsumed`) |
| APU reads | ledger `$2140` reads | replayed into the song-end polls; values never reached (`unconsumed`) or polls with none installed (`extra`) |

With `--video`, the replay first verifies the ledger's RNG sequence against the
cache's `rom-random.txt` and refuses a ledger that disagrees.

## Conventions that matter

- The engine runs Main and then the NMI that reads the pad, so engine frame k
  is handed the joypad the cartridge latched for iteration k + 1.
- A lag-free frame presents the live PPU state its own NMI wrote; the display
  snapshot's generation composition exists for lagged host frames and is
  bypassed (`LiveGpuFrameCapture::from_game_at_comparison_frame`).
- Timing-derived inputs are replayed per iteration, like the joypad: the RNG
  outputs (beam-counter seeded) and the `$2140` values the song-end polls read
  (elapsed SPC time, which lag changes). A busy-wait spins on one value for as
  long as the SPC takes, so only the sequence of distinct values is replayed
  and the last one holds; song-bank upload handshakes are not polls.
  `--live-apu-reads` lets the engine's own SPC answer instead.
- The replay renders one audio frame after each engine frame, as a frontend
  does: the audio side takes the SPC's acknowledgements, and without them the
  NMI never clears a port, so a repeated command is never sent.
- Receipt-era host-timing rules must be off under `lag_free`. Gated so far:
  the dialogue renderer's per-frame CPU budget (the cartridge finishes an
  overrunning render in the same, lagging, iteration), the interface-exit BG
  stripe deferral (the next NMI consumes it), and the Module09 sprite reload's
  deferred slot publication (an untimed reload runs in place; the untimed
  whirlpool case runs `Overworld_LoadOverlays2` to completion).

## Timing-defined sequences

Some cartridge behavior advances per NMI rather than per iteration: the
polygon thread (the intro triforce, the crystals, the ending) runs on the NMI
clock while `NMI_THREAD_ACTIVE` (`$012A`) is set, so a lagging iteration
advances it twice. The lag-free engine matches such sequences in frames, not
iterations, and its iteration count drifts there (file select appears at
iteration 807 instead of 659). The replay does not compare the thread's WRAM
while it is active and classes video misses then as `content_poly`. The
ledger also seeds the first iteration after each timing-defined stretch (the
thread running, or the intro or attract module) and lists it in
`seeds/resync.txt`; the replay reseeds there when the engine drifted or is
still inside its own stretch, so the drift ends with the sequence instead of
shifting the rest of the segment. The summary reports intro/attract segments
(modules 00, 14) apart.

## Picture oracle

`ZELDA3_LEDGER_PICTURES=1` draws every frame with the port's line renderer from
the ROM's PPU state, line by line, and writes `host-pictures.txt`; its RGB
hashes equal the Snes9x cache on most frames. `ZELDA3_LEDGER_FRAMES=lo-hi`
also writes `frames/host-<n>.ppm` and the fixed color and CGRAM, so a video
miss can be inspected as an image diff against the engine's capture
(`ZELDA3_CACHED_AV_CAPTURE_DIR` / `_FRAMES` with the replay). It is a
diagnostic: the Snes9x cache remains the authority.

## Lagless ending polygon picture

Investigation on `codex/poly1`, base `7db31d92`; CPU evidence is in
`target/poly1/`. **There are two independent effects, and the picture mismatch
is not fully resolved.**

The proven register bug is `EndSequence_32`: `$0E:BD21` loads `$12` and
`$0E:BD23` stores it to BG2SC (`$2108`). The translated initializer stored
`$13`, selecting a 64x64 tilemap instead of the cartridge's 32x64 tilemap.
This is corrected at the store. The later `$0E:BDAB..BDB0` word load/store
(`LDY #$1300; STY $2107`) really does change BG2 to 64x64; that transition
remains intact. A unit test checks both transitions against the ROM bytes.

At iteration 1,470,000, the corrected native PPU and the ledger seed agree
in all VRAM, CGRAM, OAM, BG descriptors, mode, brightness, screen enables,
color-math enable and fixed colors. Before the fix, BG2's width was their
only differing BG descriptor. Static line-renderer snapshots become identical
when that descriptor is corrected. Static snapshots omit HDMA and are **not**
cache-validated scanouts. The native software compositor, which includes the
scanline register capture, produces RGB SHA-256
`2aec28b10801343d2e7ba6a0950d478eae51b9febc4cbba126eacfceedf06dfe`, equal to
authoritative cache host 1,577,955 (one before the iteration's picture host).

The baseline worker/upload investigation also finds real phase drift. From a
single seed at iteration 1,466,000, all **1,915** distinct consecutive native VRAM
uploads equal the first 1,915 of **1,918** completed cartridge uploads, in
order, including their `$1F04/$1F05` angles. The 256 distinct bitmaps repeat
through that sequence. At every one of those 1,915 corresponding uploads,
the complete `$1F00..$1F09` inputs and `$1F3F..$1FFF` worker workspace also
agree byte for byte, including rotation products, projected vertices and
raster edges (`complete-worker-equality.json`). Comparing the logged NMI host numbers for corresponding
uploads gives offsets -1/0/+1/+2/+3 on 406/517/547/404/41 uploads. No angle or
pixel correction was applied. The native sequence ends three uploads before
the ROM sequence; equal ordered bitmaps are **not per-NMI state equality**.
The 42 initial native NMI records precede the first new bitmap and retain the
previous VRAM image. See `upload-phase.json` and `nmi-bitmap-analysis.json`.

After the width fix, the CPU compositor over iterations 1,466,000..1,473,122
finds **3,269 exact**, **1,486 within three cache hosts**, and **2,368 absent**
from cache hosts after 1,560,000. Thus a correct bitmap sequence alone does not
establish correct composite pictures: the scrolling background/fade and the
worker can occupy different phases. Full ending picture acceptance remains
open. The intro does not execute the faulty ending store; its remaining
picture misses do not share this BG2SC cause.

The intro ROM-side slice (`intro-source/`) renders 29 hosts around 172/180
and 966; **all 29 RGB hashes equal the cache at the same host**. At native
iteration 16 / oracle host 172, `$012A` is zero, BG descriptors/CGRAM/OAM
agree, and 218 VRAM bytes differ. Its failure is therefore not an active
polygon rasterizer or the ending-only BG2SC store. That separate VRAM
generation discrepancy is not repaired here.

Diagnostic additions (all CPU-only): `ZELDA3_DEBUG_POLY_NMI_STATE=<jsonl>`
records worker storage, bitmap, uploaded VRAM, palette, OAM, mode and brightness
at polygon NMI callbacks; `ZELDA3_LOGICAL_CPU_PICTURES=<path>` records RGB hashes
from the software compositor. These files append, so use fresh paths.
`--dump-wram` also saves the complete PPU JSON. The ledger example's
`--ppu-snapshot <json> <ppm>` draws a static snapshot, explicitly without CPU
or HDMA execution. None of these diagnostics changes census classifications
or the picture matcher.

Verification: the targeted ROM-witness regression test passes; default and
`--no-default-features` parity builds are warning-free. The library-test build
reports existing unrelated dead-code warnings. Both the seeded ROM census
(`--seed-every 2000`) and continuous recorded replay complete **1,473,123**
iterations and pass the unchanged gate with **unexplained 0**. ROM-lane missing
holds / excess / FP remain **2,398 / 0 / 0** (2,365 iterations have missing
holds). `git diff --check` passes. Evidence occupies approximately 255 MB.
No GPU, live replay, full library suite, protected-cache/shared-ledger write,
formatting, commit or push was performed. This is a verified partial repair,
not full ending picture acceptance.

GPU confirmation (run these **serially**; not run during this investigation):

```sh
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
L=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
mkdir -p target/poly1/gpu-saves
ZELDA3_SAVE_DIR="$PWD/target/poly1/gpu-saves" target/poly1/zelda3-default \
  --logical-replay "$L" "$C/initial.srm" --seed-every 2000 --single-seed \
  --start 1466000 --video "$C" --video-misses --report 0 \
  > target/poly1/ending-confirm-gpu.txt 2>&1
ZELDA3_SAVE_DIR="$PWD/target/poly1/gpu-saves" target/poly1/zelda3-default \
  --logical-replay "$L" "$C/initial.srm" --frames 2000 \
  --video "$C" --video-misses --report 0 \
  > target/poly1/intro-confirm-gpu.txt 2>&1
```

## Ending polygon phase

Follow-up on `codex/poly2`, base `1e7e6a81`. Evidence is under
`target/poly2/`; the cache, `full-v2`, and `target/poly1/` were read-only.
**This is a partial repair. Per-NMI upload equality and complete ending
picture acceptance remain open.** The supplied GPU confirmation is retained
at `target/ending-confirm-gpu-1e7e6a81.txt`; no GPU confirmation was run here.

Two missing main-work prices are now generated ROM block markers:

* `SaveGameFile`, `$00:894A..89C1`: **190,242 instruction clocks**, including
  128 paired SRAM-copy passes and 639 checksum words. Its continuation helpers
  partition those same blocks without replaying the prefix.
* `TransferFontToVRAM`, `$00:E556..E582`: **483,682 instruction clocks** for
  setup, 2,048 CPU word stores, and return. This is not DMA. File select's
  separately priced inline `$00:E52B..E535` copy mutes the shared helper so
  that it retains its own source price.

The worker scheduler also omitted main's recorded `$420B` DMA work from its
activation/GO positions and total work. It now includes that bus work. Before
activation, only the opening NMI uses the ordinary handler; later unfinished-main
NMIs use the retained lag handler (`$00:813A` skips updates and joypads).
Previously the worker scheduler deducted the ordinary handler on every such
frame. No polygon instruction price, `POLY_THREAD_*` constant, classifier,
scene table, or fitted threshold was changed.

These changes restore the initializer's **41** main holds (previously 39) and
its missing initial worker slice. The fresh cold CPU ROM trace shows NMI host
**1,577,170**, interrupting `$00:899D` in the checksum, returning into the worker,
then IRQ interrupting it at `$09:FB47`; main finishes on line 208. The original
native worker phase plan contained zero lag slices for that initializer.
`source/iterations.jsonl` and `source/wram-pre-nmi.xor.zst` are byte-identical
to their authoritative `full-v2` counterparts, SHA-256 respectively:

```text
9227f21db6148389146db7871447af6ab09f5a1771b15b87810b60073574a47d
b69d8c8851a1320dbf5e507045132c9054bd4db4237258c2c2a0157a084a365e
```

The residual owner is still the worker's NMI-to-IRQ scheduling and publication
phase, not its ordered raster output. The whole-worker ROM witness checks all
256 angle pairs instruction by instruction, including bus costs, divider,
workspace, bitmap, and completion flag. All **1,917** resulting native uploads
equal the first 1,917 of **1,918** ROM uploads, including `$1F00..$1F09` and
`$1F3F..$1FFF` (`dma-completed-worker-equality.json`). However, matching each
worker phase to its ROM phase gives only **2,861 / 3,908** equal upload
decisions, **828 / 3,908** equal workspaces, and **582 / 3,908** equal complete
worker cursors. Stack-frame storage is reported separately from native retained
registers; it is not silently discarded from the byte-difference receipts.

`source.log`, `source-phases.json`, `dma-phase-comparison.json`, and
`dma-workspace-comparison.json` retain the per-NMI comparisons. For example,
the first threaded lag handler takes **3,238** source clocks versus the native
constant **3,284**. An early upload handler takes **51,514** versus **51,534**.
Source NMI acceptance and instruction-boundary IRQ retirement also vary;
the native phase window still uses raster cycle 86. These observations do not
justify replacing a constant with an averaged or fitted value. At the first
remaining picture miss, source and native already disagree about whether the
previous worker slice published a bitmap.

For iterations **1,466,000..1,473,122**, `ZELDA3_LOGICAL_CPU_PICTURES` produces
**5,401 exact cache hashes**, **1,133 within three cache hosts**, and **589
absent** from cache hosts after 1,560,000. The first non-exact picture is
iteration **1,470,865**, cache host **1,578,821**. This improves the baseline
3,269 / 1,486 / 2,368 but does not meet full picture acceptance. Callback-host
offsets for corresponding uploads are -1/0/+1/+2 on 406/641/602/268 uploads;
callbacks can contain zero or two accepted NMIs, so this is not a physical
per-NMI equality proof. Pre-NMI stream image `j` closes iteration `j-1`.

A broader shared-wall-clock trial was rejected and removed: despite the same
picture improvement, its full census regressed to **2,859 missing / 437 excess /
75 false-positive iterations** by shifting crystal-cutscene work. Its patch
and logs remain labeled `shared-clock-trial.patch` / `clock-*`; its continuous
diagnostic was interrupted. They are not evidence for the retained candidate.

Verification: the retained candidate's full seeded ROM census
(`--seed-every 2000`) and continuous recorded replay each complete
**1,473,123 iterations**, **PASS**, **unexplained 0**. Final ROM missing / excess /
FP is **2,393 / 0 / 0**, with missing holds in 2,363 iterations; every runtime
hold equals P2. Predictions are identical to the pricing-only intermediate,
so the narrow worker correction adds no census regression. See `dma-counts.json`,
`dma-census-gate.log`, and `dma-continuous-gate.log`.

All **14 targeted Rust tests** and **7 block-generator Python tests** pass;
default and `--no-default-features` parity builds are warning-free. Existing
library-test dead-code warnings remain separate. The final rebuilt default
binary reproduces all 7,123 candidate CPU picture hashes byte-for-byte
(`verified-pictures.txt` versus `dma-pictures.txt`). Full gates used `dma-trial`;
the final rebuild adds only the DMA regression test and a corrected source-address
comment. `git diff --check` passes. Evidence occupies approximately **1.8 GB**.
No GPU, live replay, full library suite, protected-directory write, formatting,
commit, or push was performed.

GPU confirmation command (not executed):

```sh
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
L=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
mkdir -p target/poly2/gpu-saves
ZELDA3_SAVE_DIR="$PWD/target/poly2/gpu-saves" target/poly2/zelda3-default \
  --logical-replay "$L" "$C/initial.srm" --seed-every 2000 --single-seed \
  --start 1466000 --video "$C" --video-misses --report 0 \
  > target/poly2/ending-confirm-gpu.txt 2>&1
```

## Lagless intro picture

Investigation on `codex/intro1`, base `e38a8be3`; CPU receipts are in
`target/intro1/`. **Partial repair: the opening VRAM generation and one
visible picture are repaired. The late pictures remain unresolved.** The
cache, `full-v2`, and `target/poly1/` were read-only.

The 218-byte difference at iteration 16 / cache host 172 belongs to the
**opening NMI at host 81**, before the first main iteration. It is not
decompression or polygon content. Native logical bootstrap initialized WRAM
but omitted this handler, then main disabled core updates, preserving the
missing uploads throughout the boot card. The source's DMA register trace
(`source.log`) identifies all destinations; VRAM addresses below are bytes:

| Source instruction starting DMA | A-bus source | Differing destination bytes |
| --- | --- | --- |
| `$00:8A35`, `$00:8A59` | Six Link body/head/hand transfers from `$10:0000`, the low-WRAM mirror | 24 in `$8000..$829F` |
| `$00:8A9F`, `$00:8AD8` | Ten equipment/auxiliary transfers from `$7E:0000` | 40 in `$80A0..$83BF` |
| `$00:8AFE`, `$00:8B22` | Four pointer transfers from `$7E:0000`, plus expanded planes from `$7E:BD40` / `$7E:BD80` | 16 in `$8400..$867F`, plus 128 at `$8480..$84BF` / `$8680..$86BF` |
| `$00:8B64` | Animated BG transfer, `$7E:0000`, length `$0400` | 10 in `$0000..$01FE` |

The repeated low-WRAM bytes are `$0001=80`, `$0012=01`, `$0013=80`,
`$0015=01`. The last transfer additionally sees six nonzero live handler
stack bytes; the expanded-plane bytes retain the reset `$55` fill. Native
had zero at every one of the 218 differing destinations. The repair runs
the existing native reset/upload, memory, wait and NMI owners through the
opening handler before logical main starts. No ROM executor, recorded stack,
host-number threshold or copied oracle bytes enter production. The bootstrap
does not consume an intro work-schedule entry or logical iteration. The
existing synchronous logical audio bank remains its playback owner; the
temporary reset CPU/APU clock and scanline history retire at this handoff.

An unmodified-ROM CPU test independently executes reset through the first
NMI return and compares **all VRAM, CGRAM and OAM** with native bootstrap.
The final native VRAM also equals the source at hosts **172 and 966**.
At host 180, the source has already begun the following main's graphics
reload, so its scanout-time VRAM differs from native iteration 24's closing
state by 19,037 bytes; this is not reported as equality.

The original 218-byte discrepancy was **not the visible host-172 miss**.
After repairing it, the modern picture still had eight magenta pixels absent
from both the ROM scanout and native static line-renderer image. Its owner
was `snes9x_boot_material`: the host-timed overlay inserts stack-corner
pixels when `frame_ctr_dbg == 83`. Lagless iteration 16 happens to reach
that callback number. Lag-free capture now uses its actual PPU surface
without that host-timed overlay. The final host-172 picture is byte-exact,
RGB SHA-256 `8481fbad99d28b079d4260ec1e0223f0dc9c3e4c1965e4c2a6fd8b056b776834`.
`--dump-wram` alongside `ZELDA3_LOGICAL_CPU_PICTURES` now saves the selected
software picture as `engine-<iteration>.cpu.ppm` for pixel-level evidence.

All **2,320** fresh ROM-side CPU scanouts equal their same-host cache hashes
(`census-source/host-pictures.txt`). On iterations **0..1,999**, the modern
CPU baseline is **1,628 exact / 364 previous-iteration matches / 8 other**;
the final build is **1,629 / 364 / 7**. These are literal CPU hash counts,
not a rerun of the older GPU result **1,627 / 365 / 8**. No GPU-count
equivalence is claimed.

The reproduced late population has two owners (`late-causes.json`):

* **359 file-name scroll pictures**, first iteration **798 / host 1,094**.
  Native and ROM closing WRAM agree on `$0630=$01F0`. The *following*
  iteration's `$0C:DA98` writes `$01EF` in the preceding vblank (line 254),
  and `$00:82FB/$00:8301` consume it into BG3HOFS at the line-128 V-IRQ.
  The ROM picture therefore already contains next-main state; the logical
  capture still uses `$01F0`. Source witnesses retain every `$0630/$0631`
  write. A pre-main Link-DMA operand trial changed no picture hashes and
  was removed; the incidental 20-byte OBJ difference was not this cause.
* **Five screen-clear transitions**, first iteration **762 / host 1,057**,
  then hosts **1,579, 1,652, 1,761 and 2,215**. The following main reaches
  `$00:893F` (`STA $2100`, forced blank) / `$00:8942` (`STA $13`) before
  visible rendering. Host 1,652 reaches the store on line 1. The current
  logical closing image still shows the preceding menu. The same next-main
  blanking explains the remaining early misses at hosts **180 and 966**.

These are event-order/publication differences, not demonstrated missing
instruction prices. Replacing scroll with a one-step prediction, shifting
every capture, or fitting a scene cost would hide that ownership boundary.
No such change, new price, scene table or classifier change is retained.
The seven remaining non-exact/non-previous CPU pictures are iterations
24, 728, 765, 1118, 1186, 1465 and 1885. Full intro picture acceptance and
the exact reconciliation with the older GPU census remain open.

Verification of the retained changes: five targeted CPU unit tests pass
(opening-NMI surface equality, two reset-memory tests, NMI entry bus cost,
and logical capture's boot-overlay exclusion). Default and
`--no-default-features` product builds are warning-free; the library-test
build retains unrelated dead-code warnings. The full seeded ROM census
(`--seed-every 2000`) passes **1,473,123 iterations**, unexplained **0**.
Its missing/excess/false-positive counts remain **2,393 / 0 / 0**;
all pacing rows equal the retained `target/poly2/dma-census.csv` baseline,
and runtime held counts agree with `cost_p2` on every row. Receipts:
`rom-gate.log`, `final-pacing-comparison.json`, the `test-*.log` files and
`build-final-*.log` under `target/intro1/`. `source-identity.json` binds
the production sources, binaries and picture receipts to this verification.
The continuous recorded ROM run also passes all **1,473,123 iterations**
with unexplained **0** (`continuous-gate.log`). `git diff --check` passes;
task evidence occupies approximately **210 MB**. No GPU, live replay, full
library suite, protected-directory write, formatting, commit or push was
performed. This is a verified partial repair, not intro picture acceptance.

GPU confirmation from cold (not executed):

```sh
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
L=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
mkdir -p target/intro1/gpu-saves
ZELDA3_SAVE_DIR="$PWD/target/intro1/gpu-saves" target/intro1/zelda3-default \
  --logical-replay "$L" "$C/initial.srm" --frames 2000 \
  --video "$C" --video-misses --report 0 \
  > target/intro1/intro-confirm-gpu.txt 2>&1
```
