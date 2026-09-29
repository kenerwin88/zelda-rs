# Romless exact play: handoff

Read this first, then `docs/parity/romless-exact-play.md` for the program's
history and evidence, and `docs/parity/cycle-ledger-recipe.md` before
annotating any routine.

## Native exact CPU ownership frontier (2026-09-29)

The opt-in CPU trial now diagnoses the first differing read with the source
and translated-trial last bus writers, the pre-rebase native value, and the
active native continuation. A versioned native CPU sidecar lets paired
checkpoints replay short windows with the same ROM, source CPU, translated
trial, and writer history. `scripts/native_exact_cpu_diagnose.sh` supplies the
validated trial flags; its final argument is the absolute host at which the
contiguous translated-memory trial begins. Checkpoint replays are diagnostic,
not cold full-route A/V proof.

The first mismatch at host 13,607 was the spiral room loader's eager object
draw: source `$047E=0`, native `2`, although both CPU streams last wrote `0` at
host 13,603. The opt-in spiral load now retains the same native floor/object
cursors as the ordinary supertile room load and completes them at the source
owned caller boundary. The next two mismatches were the BG-character NMI gate
stores. The source CPU publishes `$0710=9` only after the 3/4 conversion at
host 13,627, then `$0710=10` for the 5/6 transfer at host 13,628; native had
published them before the corresponding CPU statements. Both requests are
now published by those source instruction receipts, with native values and
buffers retained.

On binary SHA-256
`5e4a299087b340242d88b489a9ee14215fa2ea766c4295acd45a4fe509922c4f`,
the resumed translated-memory CPU trial agrees on instructions, reads,
writes, and NMI schedule through host 13,632. At host 13,633 the first read
differs at `$0710`: source `0`, native `10`. The source NMI consumed request
`10` and cleared the gate at host 13,629; the native NMI dispatch did not run
that request until host 13,632. The next fix belongs to the NMI/spiral
continuation that delays this dispatch, not a forced gate clear. The same
diagnostic binary matches cached native-timing audio through frame 13,639
(`target/native-owner-final-13640.log`); no final 56,459-frame or full Metal
gate has run on it. The prior full-route A/V result used an older binary.

To reproduce from the verified frame-8192 paired checkpoint, run
`scripts/native_exact_cpu_diagnose.sh CACHE_DIR ROM_PATH PAIRED_CHECKPOINT OUTPUT_DIR 5448 11589`.
`5448` is the number of frames after the checkpoint, so this reaches frame
13,640. The script compares cached audio and traces the CPU trial; it does
not compare video. A native exact CPU run must preserve all of the script's
trial flags or it may split much earlier for unrelated reasons.

## Retained exact CPU owner in the native runtime (2026-09-28)

The next native-timing CPU frontier exposed a room-load ownership error. At
host 12,118 the source CPU and translated-memory trial both write BG2
`$7E:27D2=0x0CFD`, but the eager native `Dungeon_LoadRoom` had already
overwritten that word with an object tile (`0x8846`) before the source reached
the object loop's read at host 12,121. The opt-in supertile room load now uses
the existing native floor and object cursors and waits for the retained CPU's
actual return at `$02:8A5F`; the aggregate NMI estimate had retired this call
while source objects were still running. On binary SHA-256
`bd69c7b5528772c0ccb71841c05ff7e36fe75cad8454446784c76f8dae4050a6`,
the translated-memory CPU trial agrees on instructions, reads, and writes
through host 12,129 (`target/native-supertile-return-tail-12140.log`). Its
first schedule split is host 12,130: source Sprite_Main is held inside
`Sprite_PrepOamCoordOrDoubleRet` at the prior host return, while translated
Sprite_Main finishes a different slot prefix, leaving current sprite X
`$0FD8` at `0x0130` instead of `0x02AC`. Cached native-timing audio matches
all 56,459 contiguous frames on that binary
(`target/native-supertile-return-tail-56459.log`). The Zelda library suite
passes 1,829 tests (three ignored). A video preflight on the same binary
cannot acquire a Metal adapter (`target/native-supertile-video-preflight.log`);
the full source-route audio gate is running separately.

Cached audio-only replays now acquire no rendering comparison lock, matching
the lock contract already used by renderless live comparisons. This permits a
full source-route audio regression and a short native CPU diagnostic to run
concurrently with separate output directories.

The opt-in landing-spotlight continuation now uses the retained CPU's actual
host-return PC as well as accepted NMI positions. Source host 11,595 returns
at `$00:F38D` inside `IrisSpotlight_ConfigureTable` without accepting an NMI;
the former NMI-only cursor completed the translated builder too early. The
translated row cursor is derived from native inputs and completed row pairs,
while the source PC selects the statement boundary. A separate scheduler
gate waits for the exact CPU's main-wait return before retiring the landing
caller. Source bus observations show the reset-table stores span hosts 11,596
and 11,597, and the caller's `$012C=0x10` music command is consumed by the
following NMI. The pending music-command owner clears a stale translated NMI
sample marker only when that exact NMI reads the native command; it neither
copies source memory nor moves the audio command to an earlier host.

On binary SHA-256
`f8860cbc6fe7f143f28aa2096ac01bd975eef784bf42c0a3f51477d88a694c0b`,
56,459 contiguous cached native-timing audio frames match. The
translated-memory CPU trial has no instruction schedule split from host
11,589 through 12,120; the first split is host 12,121, beginning with a
different BG2 tilemap word read at `$7E:27D2` (`0x0CFD` source,
`0x8846` translated) at `$01:8FCA` (`target/native-music-nmi-owner-56459.log`).
The full Zelda library suite passes 1,828 tests (three ignored). These
receipts exclude video and do not establish full-route native A/V parity.

Previously frozen opt-in CPU baseline: binary
`5f5b9e33db5700e21e9e3039b7299ab4be250fcf85b7193da3a6afcb069c6063`,
first schedule split at host 11,590, 56,459 cached native-timing audio frames
matched, Zelda library suite 1,824 passed. Default source-route audio also
matches all 1,581,079 contiguous frames from zero through 1,581,078
(`target/native-predungeon-source-full.log`; video disabled). Video preflight
still cannot acquire a wgpu Metal
adapter (`target/native-predungeon-video-preflight.log`). Production native
still uses aggregate timing plans. Receipts and exact phase boundaries follow.

### Resumable room drawing and upload

The falling-entrance room loader now has native cursors for all 8,192 floor
tile stores and for complete objects in the default and three room streams.
The floor cursor computes tiles from the native asset and verifies each source
store's ordinal, address, and value; it does not copy source tile values into
native memory. The object cursor executes the existing translated object
drawers one at a time, retaining the parser section and load pointer across
CPU interruptions. Focused tests compare both cursors with the eager native
room draw at room `$0055`.

With floor and object live trials enabled, frozen binary SHA-256
`f1d9aedaf33dbbec1170da512f65ff59b438f64880c32ce15352882a0c1fa719`
matches all 56,459 cached native-timing audio frames
(`target/native-object-live-56459.log`). The exact CPU rebase trial agrees on
instructions, reads, and writes through host 8,458. The earlier tilemap read
split at host 8,457 is gone: source and translated CPU both read `$7E:2760`
as `0x10AC` at `$01:B193`. The first data split is host 8,459 at the room
quadrant-upload counter `$045C` (source `4`, trial `0`); the first CPU schedule
split is host 8,460 (`target/native-object-live-8470.log`). This remains an
opt-in translated-memory CPU trial; production native scheduling still uses
aggregate timing plans, and the receipts exclude video.

The source writes `$045C=0` at `$02:C58A` in host 8,457, then `$045C=4` at
`$00:91B3` in host 8,458; later writes advance through 8, 12, and 16. The
translated `Dungeon_UploadRoomQuadrants` was previously eager at caller
return. It is now split into native begin, per-quadrant, and finish steps,
including the VRAM upload for each quadrant. The opt-in
`ZELDA3_NATIVE_EXACT_CPU_ROOM_UPLOAD_LIVE_TRIAL=1` consumes the source
completion events as ordering witnesses. A focused test confirms that the
four-step cursor reaches the eager upload's RAM, dungeon state, and VRAM.
The Zelda library suite passes 1,824 tests (three ignored). On binary SHA-256
`4d0fa80d4e96c84ee88a3c42247faad9fb9ca7306c2ba913eae9f5bfd1e20e71`,
the exact CPU trial agrees on instruction and bus paths through host 8,461.
Its first data-only difference is a graphics-decompression buffer read in
host 8,462; that buffer reconverges by host 8,468. A later data-only split
at host 8,478 reads the misc sprite graphics index before translated
tileset initialization publishes it. The first CPU **schedule** split is
host 10,038, where source `$08:C3AC` reads ancilla step slot `$0C58=1` and
the translated-memory trial reads `3` (`target/native-upload-live-56459.log`).
That is a 1,578-host schedule advance from the prior 8,460 frontier. The
56,459-frame cached native-timing audio replay matches contiguously through
frame 56,458 (`target/native-upload-live-56459.log`). These checks exclude
video and do not establish full-route native A/V parity.

At the new host-10,038 frontier, `$0C58` is ancilla step slot 4. The source
CPU writes `1` at `$09:8801` in host 10,037 during a chest item receipt, and
both source and translated CPU see that write within the host. Native state
then reprojects the still-pending graphics continuation's older step `3` at
the next boundary. The work is `FinishItemReceiptGraphics` with a retained
`ground_apress_tail` for item `$35`; the source's tail has begun even though
the aggregate slice estimate keeps it pending. The opt-in
`ZELDA3_NATIVE_EXACT_CPU_ITEM_RECEIPT_LIVE_TRIAL=1` now runs the translated
receipt tail at the source step-store boundary and records its completion so
the later scheduler callback cannot run it twice. This is a source-ordered
native phase transition, not a copied source step value. With that trial
enabled, binary SHA-256
`5861025b346d977815162293f7d67f25c52cbb9abd5872a785c22d57e800e8ac`
matches all 56,459 cached native-timing audio frames
(`target/native-item-receipt-live-56459.log`). The exact CPU schedule agrees
through host 11,483; the next schedule split is host 11,484. Source
`Dungeon_LoadRoom` reads `$0110=0x0123` at `$01:883B` while translated memory
still has the previous room index `0x00FF`. The source header actually wrote
`0x0123` at `$01:B6C1` in host 11,480. Native is in
`FinishPreDungeonEntranceLoad { sprite_reset: Pending }` with 56 aggregate
slices left, so the room header has no translated owner yet. This is the
same class of missing early room-load publication as Module11, now under the
pre-dungeon caller. The native room cursor is now shared by both callers:
the source header event starts translated room preparation, and the pre-dungeon
return resumes the unfinished native suffix without rerunning the room draw.
On binary SHA-256
`5f5b9e33db5700e21e9e3039b7299ab4be250fcf85b7193da3a6afcb069c6063`,
the opt-in CPU path agrees through host 11,490 and 11,500 cached
native-timing audio frames match (`target/native-predungeon-live-11500.log`).
The 56,459-frame cached native-timing audio replay also matches
(`target/native-predungeon-live-56459.log`). The first CPU schedule split is
host 11,590, where `IrisSpotlight_BuildTable` at `$00:F36D` reads the
spotlight window Y buffer `$067A=5` in source and `0` in translated memory.
The first intervening data-only split after this room load is host 11,507:
the source's misc sprite graphics index has changed, but translated tileset
initialization is still deferred. A read-only `$067A` trace across hosts
11,580–11,590 (`target/native-spotlight-owner-watch-11591.log`) identifies the
next owner: native enters `Module07_0F_LandingWipe` and schedules
`FinishDungeonAfterSubmoduleCallerReturn`, while the source has already run
the spotlight prologue at `$00:F341` and is iterating its table builder at
`$00:F36D`. Source writes successive window Y values 70, 77, 84, 91, and 98
across hosts 11,581–11,589; native still presents `$067A=0` at host 11,590.
The next clean phase is a source-ordered native spotlight builder cursor,
using `begin_iris_spotlight_configure_table_at_progress` and retaining its
table, HDMA, and caller-continuation state. A direct `$067A` correction would
hide the missing phase. The full source-route audio replay on this binary
matched all 1,581,079 cached frames in 1,024.92 seconds. Default production
timing remains aggregate.

### Interrupted sprite preparation and next room-load owner

The final frozen trial binary is SHA-256
`a84df260e2d461feccd89e703c34f27763a012fc2ac3ad3f37bf5aa1d07a0521`.
The Zelda library suite passes 1,821 tests (three ignored). With the exact
CPU owner, live scan, sprite-preparation, and early room-header trials enabled,
it matches 56,459 contiguous cached native-timing audio frames
(`target/native-sprite-refresh-56459.log`). With the trials disabled, the
same binary matches all 1,581,079 cached source-route audio frames,
contiguous from zero through 1,581,078
(`target/native-sprite-refresh-source-full.log`). These checks exclude video.
A one-frame video preflight failed to acquire a Metal adapter from wgpu
(`target/native-sprite-refresh-video-preflight-final.log`), although macOS
reports an Apple M2 Max with Metal support; no video or production native
A/V parity claim follows from these audio results.

The opt-in `ZELDA3_NATIVE_EXACT_CPU_SPRITE_PREP_LIVE_TRIAL=1` now consumes
the retained CPU's instruction-ordered `NMI_PrepareSprites` packing cursor.
When an NMI interrupts the Module0E caller, translated gameplay publishes
only the bytes completed before acceptance, keeps the shared suffix pending,
and returns through that suffix after the carried handler completes. The
source CPU also records the `$00:805D` main-wait latch release, so the return
cannot be inferred from a host-frame count. At host 7,322 the interrupt is
accepted at `$00:8605` after 88 master cycles in packing group 16; no group
byte has completed. Host 7,323 finishes that NMI, and the caller clears `$12`
at master cycle 2,616,952,186.
The cursor counts charged instruction transactions rather than physical
timestamp deltas. WRAM refresh can add physical elapsed cycles without
advancing the OAM store instruction, and interrupt-entry clocks belong to the
handler. A later interruption around frame 14,169 exposed the distinction;
the full 56,459-frame trial now passes with the nominal cursor and a focused
refresh/interrupt-entry test.

With the live scan and sprite-preparation trials enabled, the contiguous
translated-memory CPU's first **schedule** split moves from host 7,324 to
8,455 (`target/native-sprite-prep-live-trial-10000-c.log`), a 1,131-host
advance. All 10,000 cached audio frames match. At host 8,455, source
`Dungeon_LoadRoom` reads `$0110=0x00ff` at `$01:883B`; the translated-memory
trial supplies the stale `0x030c`. The following branch at `$01:8AA1` takes
a different cycle count and admits an extra NMI in the trial. Earlier
data-only mismatches can occur without changing the CPU schedule; the
contiguous trial retains them rather than claiming full read equality.

The separate `ZELDA3_NATIVE_EXACT_CPU_ROOM_HEADER_LIVE_TRIAL=1` consumes the
source header's actual `$01:B6C1` store to `$0110` and publishes the whole
translated `Dungeon_LoadHeader` phase on that host. The ordinary room load
still calls the same header in its original order. In the trial the saved
HDMA mask is restored only after the deferred room draw, and the header is
not replayed at completion. Source room `$0055` produces index `$00FF`,
matching the translated entrance room; the source write is in host 8,451,
before the room-draw NMI. This removes the `$0110` schedule split. The first
source/translated read difference in that room load is then host 8,457 at
BG2 tilemap `$7E:2760` (`0x10AC` versus `0x046E`), while the first schedule
split is host 8,458 on the final binary
(`target/native-sprite-refresh-cpu-8460.log`). The
remaining aggregate 56-slice continuation defers floor and object drawing;
the next clean step is a source-event-driven resumable floor/object cursor,
with intermediate tilemap writes owned by the translated room draw. Copying
source tilemap words into the trial would hide the missing execution phase.

`ZELDA3_NATIVE_EXACT_CPU_WATCH_WRAM_ADDR` is an opt-in hexadecimal bus watch
for tracing source and translated-trial reads and writes to a WRAM address.
It records instruction PC, width, value, and master-cycle order without
publishing source memory to native gameplay.
The contiguous CPU trial now compares bus writes as well as instructions and
reads, so a write divergence cannot be reported as an identical path merely
because the changed value is read in a later host.

### NMI branch ownership at the dialogue frontier

The retained CPU now records bus-ordered `$12`/`$0710` accesses with NMI
ancestry and emits a typed decision at the ROM's `$00:8138` latch read. This
is opt-in observation; it does not yet drive translated NMI scheduling.
At source host 7,322, RenderText writes `$0710=2` repeatedly before an NMI
is accepted at master cycle 2,616,940,768. That handler's `$12` read in host
7,323 sees `1`, so it skips `NMI_DoUpdates` and never reads `$0710`. The next
NMI, accepted in host 7,324, reads `$12=0`, then `$0710=2` at `$00:89E7` and
clears it later in the same handler. See
`target/native-nmi-gate-ordered-cpu-10000.log` and the typed event tests in
`rtl_native_exact_cpu.rs`.

The translated handler already sees its latch set on native frame 7,323,
but takes the following leading NMI on native frame 7,324, which corresponds
to source host 7,323. That consumes `$0710` one host before the source's
acceptance. The clean next change is to let source NMI acceptance and the
`$12` branch decision own a retained translated handler/caller boundary;
audio, DMA, and main-loop execution must stay ordered around that boundary.
A trial that merely deferred a RenderText trailing NMI did not run at this
point because the scheduler was already in the leading-NMI phase; it was
removed. A broad suppression of NMIs also failed audio at host 4,926 and
remains removed. No native CPU or A/V frontier advance is claimed here.

The frozen diagnostic binary has SHA-256
`4acaaac3f29dc6149777058ac4961d544d544ac23f2dfc8023ae2a677711118c`.
On it, 1,819 Zelda library tests pass (three ignored), the opt-in live scan
matches 56,459 contiguous cached native-timing audio frames
(`target/native-nmi-gate-ordered-native-56459.log`), and the complete cached
source route matches 1,581,079 contiguous audio frames
(`target/native-nmi-gate-ordered-source-full.log`). The translated-memory CPU
trial still first splits at host 7,324 on `$0710`, with source value `2` and
translated value `0` (`target/native-nmi-gate-ordered-cpu-10000.log`). All three
comparisons exclude video, and source-route audio preservation does not prove
production native A/V parity.

### Source-driven live overworld scan trial

`ZELDA3_NATIVE_EXACT_CPU_SCAN_LIVE_TRIAL=1` with
`ZELDA3_NATIVE_EXACT_CPU_OWNER=1` now keeps the translated Module09 reload's
presence publication and proximity scan as separate live phases. The source
CPU's `$09:C56A` write starts the translated cursor, each `$09:C5E6` return
advances it once, and `$09:C585` finishes it. A detached preview supplies
the existing aggregate workload and deferred final slot generation while this
is an opt-in trial; the source event drives the CPU-visible scan state, without
copying source WRAM. This is not yet the default scheduler or a completed
replacement for the aggregate publication plan.

On the frozen binary
`f2608ba07ce17967b9824d065383d2439bd7a0d41f1d56ebf9f732503fd2e264`,
the live trial matched 56,459 contiguous cached native-timing audio frames.
It completed four source-driven scans, including the 6,949/6,950 and
39,267/39,268 split scans (`target/native-live-scan-final-native-56459.log`).
The same frozen binary matches all 1,581,079 cached source-route audio
frames, contiguous from 0 through 1,581,078
(`target/native-live-scan-final-source-full.log`); the Zelda library suite
passes 1,816 tests (three ignored).
The contiguous translated-memory CPU trial now remains instruction-, read-,
and clock-identical through host 7,323; its first instruction split is at
host 7,324 on `$0710` in the NMI handler
(`target/native-live-scan-final-cpu-10000.log`). The source did not accept
an NMI in host 7,323, while native consumed the pending dialogue command in
that host. A broad experimental suppression of native trailing NMIs was
removed: it broke audio at frame 4,926 even though the CPU instruction trial
remained aligned there (`target/native-exact-nmi-authority-phase-cpu-10000.log`).
The next clean owner must coordinate the source NMI acceptance, translated
caller phase, and APU publication; copying `$0710` back would only mask the
phase error. A one-frame video preflight still found no Metal adapter
(`target/native-live-scan-final-video-preflight.log`), so this trial has no
new video or production A/V frontier claim.

The Module09 loader now exposes a `OverworldSpriteReloadScanWork` caller that
begins after slot reset and overworld presence publication, advances to an
exact completed-cell ordinal, and finishes only after all cells. Ordinary
gameplay still completes it in one translated call. With both
`ZELDA3_NATIVE_EXACT_CPU_OWNER=1` and
`ZELDA3_NATIVE_EXACT_CPU_SCAN_TRIAL=1`, an opt-in shadow caller consumes the
CPU owner's actual scan events while the live caller keeps its existing
publication behavior. The shadow retains its `$069F=$FF` scan value after
347 cells at host 6,949, then returns after cell 484 at host 6,950. At that
return its full translated 128 KiB RAM and `GameState` equal the ordinary
eager scan's post-return state, including sprite slots, presence, loaded
masks, and caller scroll restoration. Across 56,459 cached native-timing
audio frames, four scans finished with identical full translated state; two
crossed a host boundary (6,949/6,950 and 39,267/39,268). All 56,459 audio
frames matched contiguously (`target/native-exact-scan-shadow-final-native-56459.log`,
binary SHA-256 `e40aad955f5633142d66009650021180e07ec2f04bbed971f1eb13f867325c71`).
The same binary matched all 1,581,079 cached source-route audio frames,
contiguous from 0 through 1,581,078
(`target/native-exact-scan-shadow-final-source-full.log`). The Zelda library
suite passes 1,816 tests (three ignored).
This validates the whole resumable caller's end state under source CPU event
ordering; it does not make the shadow authoritative or advance native A/V
parity. The live Module09 publication still needs to move from the aggregate
timer to the retained CPU events, with OAM and slot generations reconciled at
each NMI boundary. The final binary's contiguous translated-memory CPU trial
remains instruction- and clock-aligned through host 6,949; host 6,950 first
splits on the `$069F` read (`target/native-exact-scan-shadow-final-cpu-6951.log`).
The shadow's `$FF` at that boundary identifies the missing live handoff, not a
reason to exclude the byte from the comparison.

`ZELDA3_NATIVE_EXACT_CPU_OWNER=1` now retains a cold, source-ordered Rust
65816/PPU/APU CPU owner across native host calls. It takes the same raw host
input and records accepted interrupts and beam-register reads, but does not
publish source WRAM or display state to the translated game. It is an opt-in
integration probe, not a production timing replacement or native A/V proof.
The production scheduler still uses aggregate `RomCpuTimingRun` plans.

The central CPU owner now emits typed overworld proximity-scan boundaries
from the original ROM's own instructions: the temporary `$069F` write at
`$09:C56A`, each return from the `$09:C6F5` per-cell call at `$09:C5E6`,
and the restore at `$09:C585`. On the recorded route it observes a begin and
347 completed cells in host 6,949, then 137 more cells and a finish at ordinal
484 in host 6,950 (`target/native-resumable-scan-final-events-6949.log`,
`target/native-resumable-scan-final-events-6950.log`). The translated
`sprite_activate_all_proxima` now uses a resumable cell cursor. Its ordinary
caller runs that cursor to completion; a future CPU-owned caller can advance
it through a source cell ordinal without replaying completed cells or copying
source WRAM. The central event and translated cursor are not yet connected in
production. The current Module09 loader still precomputes the scan in one
translated call, so these changes do not claim a later native A/V frontier.
The remaining handoff is concrete: park `Module09_LoadNewSprites` after its
reset and presence publication, retain the cell cursor as part of the
suspended translated caller, consume source `CellReturned` ordinals before
each NMI publication, and retire it only on the source `Finished` event.
Sprite activation and OAM publication must follow the same cursor; merely
holding `$069F=$FF` after the eager full scan would preserve the wrong
execution order. Once this caller owns its source-timed scan, its aggregate
`load_nmi_slices` estimate can be removed and the CPU trial rerun.

The current binary is
`d98b23f7ae273596c316254e6e0c1c1094f251adb94b83a4788f9fc69ce27a7d`.
It matches 56,459 contiguous native-timing audio frames
(`target/native-resumable-scan-final-native-56459`) and all 1,581,079 cached
source-route audio frames, contiguous from zero through 1,581,078 with no
RNG drift (`target/native-resumable-scan-final-source-full`).
The opt-in retained CPU owner also completes the same 56,459-frame native
audio window without RNG drift or scan-event lifecycle failure
(`target/native-resumable-scan-final-owner-56459`). A clean rebuild after
removing unrelated formatting changes reproduces the exact binary SHA-256.
At native frames 6,948 through 6,951, its full 128 KiB WRAM images are
byte-identical to the previous binary's captures. The exact CPU trial still
splits at host 6,950 on `$069F`
(`target/native-resumable-scan-final-cpu-10000.log`): the new
cursor preserves current behavior but is not yet a production scheduler
handoff. The Zelda and SNES library suites pass 1,815 and 497 tests. A
one-frame native video preflight again failed before rendering because wgpu
found no Metal adapter (`target/native-resumable-scan-final-video-preflight.log`).

The retained owner can now rebase its private memory at a **quiescent
instruction boundary** inside a host call, rather than only at host entry.
The diagnostic trial waits for the `RTI` of the specific NMI carried into the
host before copying translated-owned memory; it retains the physical clock,
PPU/APU/DMA bus, pending interrupts, and CPU-private stack and scratch. This
keeps the source instruction schedule aligned at host 6,174, where a generic
host-entry rebase had broken it. With the source-owned bank-`$7F` overworld
decode buffers (`$4000..$41FF` and `$4400..$44FF`) excluded from that copy,
the contiguous trial remains instruction- and clock-identical through host
6,949. The trial then splits at host 6,950 on a read of `$069F`: source reads
`$FF`, translated memory supplies `$00`. See
`target/native-exact-cpu-quiescent-hook-7000.log`,
`target/native-exact-cpu-overworld-scratch-10000.log`, and
`target/native-exact-cpu-scroll-delta-source-6950.log`.

`$069F` is the horizontal scroll delta, not CPU-private decode storage. The
source writes `$FF` at `$09:C56A` during host 6,949's suspended overworld
sprite proximity scan and clears it in host 6,950. The source's host-6,949
semantic receipts include `PresencePublished` and `ProximityScanSuspended`,
while the native frame-6,949 WRAM capture already holds `$00` at `$069F`
(`target/native-scroll-delta-6951`). At CPU host 6,949 the native scheduler is
already in `FinishOverworldSpriteReloadTail` with two NMI slices remaining,
while the source is still publishing presence and suspending the scan
(`target/native-scroll-delta-scheduler-6949.log`,
`target/native-exact-cpu-scroll-delta-semantic-6950.log`). The aggregate plan has
not retained the source scan's in-flight scroll-delta value in translated
memory; the scheduled tail by itself does not prove that the scan has returned.
The central CPU trial deliberately stops at that read; excluding `$069F`
would mask the problem. The next production step is to drive the suspended
scan's publication and return from source-ordered CPU events instead of a
whole-reload estimate, then rerun the contiguous CPU trial and A/V gate. None
of these diagnostic rebases changes normal gameplay.

The earlier quiescent-trial binary SHA-256 is
`1cc20272d86a062cbf0554bdfd1bffb8e37e9023f7074b5981129af919f25537`.
It matches 56,459 contiguous native-timing **audio** frames with no RNG drift
(`target/native-exact-cpu-quiescent-final-native-56459`); the Zelda library
suite passes 1,813 tests and the SNES library suite passes 497. The same
binary matches all 1,581,079 cached source-route **audio** frames, contiguous
from zero through 1,581,078 with no RNG drift
(`target/native-exact-cpu-quiescent-final-source-full`). A one-frame native
video preflight failed before rendering because this process still reported no Metal adapter
(`target/native-exact-cpu-quiescent-final-video-preflight-native.log`).

The owner runs contiguously through native host 56,458 with audio parity.
On host 56,457, its retained PPU bus reads `$06:2137` at master clock
20,175,926,238 and `$06:213C` at 20,175,926,268; the latter returns `$EA`,
the source high-byte/open-bus result that the fresh native shadow loses.
On host 56,458 its exact NMI acceptance is at master clock 20,176,119,184,
inside `$06:DCE3` (the PC after the interrupted opcode). The native timing
shadow independently reaches `$06:A618`; this is the same CPU ownership gap
identified by the source trace, now visible from a retained in-process owner.
The central CPU receipt records the interrupted PC before entering the NMI
handler, separately from the opcode's starting PC. A branch or multi-byte
opcode makes those different, so a continuation must use the former.

Source and translated WRAM captures at the host returns differ in 42 bytes at
56,457 and 108 bytes at 56,458. These include scroll inputs and sprite tables,
not just the CPU's private scratch. Copying the source CPU's NMI timestamp,
register state, or whole WRAM into the translated scheduler would therefore
hide the execution mismatch. The next production step needs one bus transaction
owner that charges translated semantic operations in source order and carries
PPU read phase through ordinary main waits and NMIs. The exact sidecar supplies
a diagnostic timeline but does not yet perform that handoff.

Validation for the working binary SHA-256
`d5ae40e9310e8051fb407f84d1b73400f044d340bc4cc602fbdcf2f5a51d649e`:
the opt-in native run matched 56,459 contiguous audio frames and observed the
correct `$06:DCE3` NMI PC at host 56,458. The source-route comparison matched
all 1,581,079 contiguous **audio** frames in
`target/native-exact-owner-full-audio-final`. Video remains unverified on this
binary: even a one-frame preflight failed because wgpu found no Metal adapter
(`target/native-exact-owner-video-preflight.log`). Do not report this as an
exact 1.5M A/V gate or a new production-native frontier.
The last measured translated-native video frontier remains frame 56,458 on
the earlier validated binary.

The exact CPU executor now has a quiescent *private timing shadow* rebase.
A future translated timing plan can supply its program registers and copies of
its authoritative WRAM/SRAM at an exact master-clock timestamp. The executor
retains its physical timeline, pending interrupts, PPU read bus, APU, DMA, and
CPU OpenBus. It never publishes private source memory back to native gameplay.
The rebase rejects a pending bus completion, suspended CPU state, wrong clock,
or wrong memory length before mutation. A synthetic test retargets the next
opcode to `LDA $213C`, reads the retained `$EA` high phase, then reads the
translated WRAM byte from the private copy. This is a prerequisite for
replacing a ROM timing plan; no production plan invokes it yet.

The handoff cannot be attached to a frame number. Before native host 2,337,
the retained source CPU is at `$00:80C9` (NMI handler entry), while native's
main-wait phase is already recorded for host 2,338 at `$00:8036`; their WRAM
images differ in 1,727 bytes. At the *end* of host 2,337, both CPU paths have
reached `$00:8036`, but their WRAM still differs in 1,728 bytes. Before host
56,458, source is at `$06:DCE2` inside a suspended sprite draw, native has no
main-wait phase, and their WRAM differs in 1,785 bytes; at the host return,
source reaches `$00:8036` and 1,863 bytes differ. Both lanes report the same
main module. These are different game-state representations, not a safe
whole-WRAM equality or copy boundary. Evidence is in
`target/native-exact-owner-entry-2337`,
`target/native-exact-owner-after-2337`,
`target/native-exact-owner-entry-56458`, and
`target/native-exact-owner-after-56458`. The temporary comparison code was
removed after recording the results. A production caller must align the same
instruction transaction timestamp, seed the timing shadow from translated
state, and account for its plan-specific ROM stack/asset normalization.

The intermediate binary SHA-256
`c1bd485d6af2f0de23470ea363a0c9f6fad3c6eabcafc7bd169b779e6914f894`
matched 56,459 contiguous native audio frames with the opt-in owner
(`target/native-exact-owner-rebase-audio`). The source NMI at host 56,458
remained `$06:DCE3` at master clock 20,176,119,184. The new rebase API is not
called by the production scheduler, so neither that run nor the later audits
establish a new video frontier or a fresh full-route gate.

The checkpoint's program-register mapping is now shared by the production
`RomCpuTimingRun::new` constructor and any future exact-owner rebase. This
removes a second source of CPU-register drift when switching a timing plan.
The current binary SHA-256
`2f8f0b464545aa06ec2d6d72f5fcf3e05e6024a49ad7db25db2e650138946209`
matched 56,459 contiguous native audio frames in
`target/native-private-shadow-prefix`; the exact sidecar still observed the
source `$06:DCE3` NMI. All 495 active SNES library tests, the focused Zelda
timing test, and the Zelda library check pass. The private shadow rebase is
not yet called by native scheduling, and it does not reconcile translated
PPU display/DMA control state or plan-specific stack/asset normalization.
The full 1,581,079-frame audio proof above remains on the earlier binary.

The native timing-plan profiler now accepts
`ZELDA3_DEBUG_ROM_CPU_PROFILE_HOST=<host>` or `<first>-<last>` alongside
`ZELDA3_DEBUG_ROM_CPU_PROFILE=<dir>`. It selects plans before allocating their
per-instruction attribution, so a long route can inspect one handoff window
without generating profiles for every host. An unrestricted 56,459-host
attempt produced about 1.6 GiB of profiles and exhausted available disk
space before replay completed; those generated files were removed. Profile
output is diagnostic evidence, not a production CPU handoff.

With the bounded profiler, binary SHA-256
`d57b1c6cfe579a021d6d1ce6914c94924c63dbce9dca44e32896fd474850f70e`
matched 56,459 contiguous native **audio** frames in
`target/native-plan-window/result`. The seven timing plans selected for hosts
56,450–56,459 are in `target/native-plan-window/host-*.json`. Host 56,458's
fresh shadow begins at `$00:8036`, stops at `$00:805D`, and executes 11,917
instructions / 337,406 master cycles with one NMI. The retained exact CPU is
already at `$06:DCE2` before that host, so this fresh main-loop plan is not a
valid exact-owner rebase point. The next owner change must carry CPU and bus
state through the earlier translated operations that led into this suspended
routine, rather than switch at the host number or replace only this plan.
The one-frame video preflight on this same binary still fails before comparison:
wgpu Metal reports `no suitable GPU adapter: NotFound`
(`target/native-video-preflight-current.log`). Its 56,459-frame result is
audio-only and does not advance the measured video frontier.

An opt-in rebase trial now clones the retained exact CPU at an arbitrary host,
imports translated WRAM/SRAM into that *copy*, and compares source-ordered bus
reads, NMIs, final PC, and clock with the untouched owner. A full-memory copy
at host 2,337 first changes the CPU return-stack read at `$00:822C` from
`$32` to `$00` (`$01FC`), then reaches invalid PC `$000000` after only 240
reads. The same trial preserving the active stack segment reads all 18,237
bus values at the same timestamps and finishes at the same `$00:8036` and
clock 835,471,248. This shows why CPU-private stack bytes cannot be
overwritten by translated WRAM even when game state is rebased. The central
executor now supports validated, sorted, non-overlapping translated-memory
regions, with a synthetic `JSR`/`RTS` test proving return-stack retention.
The stack-segment selection in the runtime trial is diagnostic; a production
caller must prove its complete active stack extent.

A bounded trial over hosts 2,293–2,340 confirms this is a conditional
boundary, not a blanket frame-level rule: 24 of 48 independent one-host
stack-preserving trials follow the same read/NMI/PC/clock path. The early
sequence alternates between matching main waits and mismatched poly-thread
hosts; the first mismatch at 2,293 is `$12` (`0` source versus `1`
translated), and later poly-thread trials first disagree on private `$02/$04`
words. Hosts 2,335–2,340 match consecutively. See
`target/native-exact-rebase-trial/early-range.log`. Any production takeover
must choose a proved instruction transaction and memory ownership map, not
enable the rebase on every host return.

At host 56,458 the stack-preserving trial still first diverges at the NMI
gate's `$00:8138` read of `$0012`: the exact owner reads `1`, translated
prehost WRAM supplies `0`, at the same master clock 20,176,120,194. The trial
then accepts an extra NMI and ends at `$00:80C9` instead of the ordinary host
return. This is a temporal NMI-handshake mismatch, not a stack-byte or Cucco
offset to patch. The full and stack-preserving receipts are in
`target/native-exact-rebase-trial`. No trial result drives production native
timing or publishes source memory into translated gameplay.

The 56,400–56,458 scan finds 45 matching and 14 mismatched independent
stack-preserving one-host trials. The failing hosts are 56,412, 56,417,
56,421, 56,423, 56,425, 56,427, 56,439, 56,442, 56,445, 56,448,
56,451, 56,454, 56,456, and 56,458. Most first mismatches are low
direct-page words `$00/$02/$0A`; 56,458 is `$12`. Host 56,457 matches, but
this intermittent result cannot license a host-number switch or a blanket
direct-page copy. The exact owner and translated scheduler need a shared
transaction-level account of which NMI/control and game-state writes have
committed before each read.

Retaining the CPU's current direct-page window as well as its active stack
segment makes 58 of those 59 **independent** trials instruction-compatible.
The sole remaining mismatch is host 56,458: after the correct `$06:DCE3`
NMI, the exact CPU reads Cucco slot 2's `$0F52` at `$06:A620`. The source
value is `$49`; translated prehost WRAM supplies `$59`. The source only writes
`$59` at `$06:A625` after it resumes the interrupted shadow draw. Source
`$06:DCE0` already wrote the shadow OAM flags before NMI, while its extended
OAM store at `$06:DCEC` and the Cucco `$0F52 |= $10` suffix remain pending.
The translated Cucco routine currently completes that suffix before the
source CPU's NMI boundary. This identifies a real semantic publication-order
defect, not a missing fixed cycle charge. The independent diagnostic receipts
are in `target/native-exact-rebase-trial/frontier-dp.log`; source writes and
the shadow-draw instruction tail are in `source-oam-flags.log` and
`source-shadow-tail.log` beside it. A contiguous translated-memory trial is
still needed before using this CPU as the production timing owner.

The native OAM bridge now exposes the ordinary-entry write and its following
extended-OAM byte as separate operations; the existing atomic helper composes
them in the same order. This is the bus-level seam needed for the NMI at
`$06:DCE3`: source has published shadow X/Y/character/flags through
`$06:DCE0`, but its `$06:DCEC` extended byte is pending. A focused native
test and a sprite-draw test check the partial publication and exact completed
state; both pass. No native
scheduler invokes the partial helper yet, so this does not claim a frontier
advance.

The contiguous stack/direct-page trial from host 56,400 matches every bus
read, accepted NMI, final PC, and master clock through host 56,457. Its first
failure is again the `$0F52` read at `$06:A620` on host 56,458
(`target/native-exact-rebase-trial/contiguous-56400.log`). This rules out a
hidden accumulation error in that late-game window: the exact owner carries
the correct physical PPU/NMI phase across 58 successive translated-memory
rebases. A much earlier contiguous trial from host 2,335 matches 90 hosts,
then fails at host 2,425 when the source reads the dialogue message-source
offset `$1CDD` as `$0050` but translated WRAM supplies `$0000`
(`contiguous-2335.log`). The region mapping is therefore not safe across all
modules; dialogue pointer representation needs explicit ownership before a
route-wide takeover. Neither trial changes the production native schedule.

The diagnostic now distinguishes exact bus-value parity from exact
instruction timing. Original dialogue decoder offset and pointer-table bytes
stay in the CPU-owned region because the original ROM and translated dialogue
assets encode them differently. With that semantic ownership, a contiguous
trial from host 2,335 matches every read and instruction through host 2,506,
including the former `$1CDD` failure at 2,425. Host 2,507 first reads a
different `$7F:0002` tile-buffer word (`$48B5` source, `$48B4` translated),
but all 8,510 executed instructions and both NMIs still have identical PCs
and master-clock timestamps. Retaining a trial on data-only differences shows
the instruction schedule remains exact through host 3,816. At 3,817 a VWF
read at `$0E:CCBD` first sees `$40` source versus `$00` translated in
`$7F:01C5`; the subsequent glyph-column branch at `$0E:CC32` takes a
different number of master cycles. This identified a source-backed ownership
gap: dialogue's mutable tile buffer must be published at the same CPU
transaction point as the renderer, not simply exempted from rebasing. The
receipts are `target/native-exact-rebase-trial/contiguous-schedule-10000.log`
and the source instruction traces `source-2507-7f0002.log` and
`source-3817-vwf.log`. The 10,000-frame cached replay used for this
diagnosis matched audio only; it did not change production native timing.

The frozen intermediate binary SHA-256
`3e87fe2e33b3e5f30286e91c5a9736e46fb9ee0f572aadf4f5eed10719308f93`
matched all 1,581,079 contiguous source-route **audio** frames in
`target/native-current-full-audio`. Its manifest identifies zero video frames.
The instruction-timeline diagnostic and dialogue ownership extension were
built after that run, so this full-route receipt must not be attributed to
the final working-tree binary.
The later binary SHA-256
`002d6e8dab86e937d001613b43c880d4d3c3b92658dc3b28532db1f6c28448f6`
includes instruction-level schedule comparison and matched the 10,000-frame
diagnostic audio replay. Its video preflight still cannot start because wgpu reports
no Metal adapter (`target/native-current-video-preflight.log`).
The same final binary passed all 1,581,079 contiguous **source-route audio**
frames in `target/native-final-source-full-audio`, with frames 0–1,581,078
paired and no enabled video lane. This gate did not enable translated native
timing; the native result remains separate.
With translated native timing enabled, that binary also matched 56,459
contiguous audio frames (`target/native-final-56459-audio`). Extending the
same native run exposed a ROM-random call-order divergence at execution frame
56,482 in `cucco_summon_avenger`: Rust called while the replay expected frame 56,488
(`target/native-final-full-audio.log`). This is beyond the measured video
frontier and may be downstream of the unresolved CPU/OAM continuation; it is
not a basis for a frame-specific random fix. A fresh full source-route audio
gate on the final binary does not erase this native timing failure.

### Transaction-ordered VWF stores (later working tree)

The source writes `$7F:01C5 = $40` from `$0E:CCB7` during host 3,816 and
returns inside the lower half of the glyph at `$0E:CC9B`. The earlier
translated renderer held the whole glyph bitmap until completion, leaving
that byte `$00` at the host return and publishing final `$49` on the next
host. The VWF cycle model now emits the actual byte/seed-word stores in
instruction order. Native rendering applies each store only when its CPU
drawing phase crosses that store, and the old atomic result is unchanged when
the glyph completes. The ROM's `$0724` glyph cursor changes on drawing entry;
`$1CD9` changes after `$0E:CCF1 INC` but before the remaining glyph epilogue.
The native continuation retains its original glyph decoder position across
that early `$1CD9` store so resuming does not render the next glyph twice.
The synthetic bitmap replay test passes, as do the ROM-backed 1,904-glyph
and 71-command cycle-model comparisons and the VWF runtime tests.

With this ownership, native WRAM at the 3,816 return has `$7F:01C5 = $40`
and `$0724 = 12`, and the retained CPU trial matches the source's reads,
instructions, NMI, PC and clock through host 3,817. The same checks match at
the former `$1CD9` mismatch on host 4,654. A contiguous trial from 2,335
has the same **instruction schedule** through 4,858. Its first schedule split
is now at host 4,859: `$09:C6FA` reads `$0FBC` as `$0800` in source timing
but `$0000` from translated WRAM. Source `$09:C4B5` wrote the high `$08`
byte during the preceding host inside `Overworld_LoadSprites`; translated
pre-overworld reload work has not published that collision-base prefix yet.
This is a lifecycle stage of a suspended overworld call, not a reason to
preserve one byte by frame number. Trial receipts are in
`target/native-vwf-readpos-10000.log`, with source writes in
`target/native-exact-rebase-trial/source-4859-0fbd.log`.

The working binary SHA-256
`1db53bbac362da07f75bfc9b8a632db30b0564d19136d31ce816b744d2a58045`
matched 56,459 contiguous translated-native **audio** frames in
`target/native-vwf-final-56459`; video remains unverified because the Metal
adapter preflight failed on the preceding binary. A fresh one-frame preflight
on this binary also failed before comparison with `no suitable GPU adapter:
NotFound` (`target/native-vwf-current-video-preflight.log`). This does not yet advance
the production translated-native frontier at host 56,458.

The same frozen binary passed the complete 1,581,079-frame **source-route
audio** cache in `target/native-vwf-final-source-full` (manifest SHA matches,
frames 0..1,581,078 contiguous, no RNG drift). That gate exercises the new
code without establishing translated-native timing parity.

The focused translated-native replay `target/native-vwf-4859-schedule` confirms
that at frame 4,859 the scheduled C caller is still
`FinishPreOverworldProperties { sprite_presence_published: false }` and
`$0FBD` is zero. The source CPU committed `$0FBD=08` at `$09:C4B5` during
host 4,858. The next clean ownership step is to expose the entry/prefix of
`Overworld_LoadSprites` within that suspended call, so the translated
collision bounds are published when the CPU reaches their stores. Moving a
single byte or tuning the call's completion frame would hide the cause.

### Staged native sprite reload (next working binary)

The native scheduler now gives `FinishPreOverworldProperties` an explicit
presence-loaded stage. At its first held NMI, `overworld_load_sprites()`
publishes the collision bounds and presence map, and the proximity scan's
temporary scroll state begins. The scan and caller suffix still complete at
the scheduled return. Source-receipt execution retains its separate
activation-applied stage. This models the suspended C call's semantic order,
not a special case for route frame 4,859.

On binary `5c3e10d4ee09b280712cae02cd6058c0062b9f85ccc6edd4a961f82653d04964`,
the contiguous CPU trial from 2,335 matches source reads, instructions, NMI,
PC and clock through host 4,861. The former `$0FBC` and `$069F` divergences
are gone. Native audio remains exact through frame 56,458
(`target/native-preow-staged-56459`). The next schedule split is host 4,862:
source `$02:F695` reads live Map32 decode scratch `$7F:4440=$3FD0`, while
translated memory still has `$04FB` because `PreOverworld_LoadOverlays` holds
the entire quadrant decode until its caller return. Source
`target/native-preow-map-source-window.log` shows `$02:F695` entering 14
Map32 definitions during host 4,861 and 103 during 4,862, with the cache
last-value word updated at `$02:F69D`. The next structural fix is a typed
quadrant decoder that publishes each definition's C stores according to the
central CPU timeline, including decompression before the first tile and the
cache reset between quadrants. Publishing the final map early would skip the
observed partial-state boundary.

### CPU-counted overlay quadrant (subsequent working binary)

The pre-overworld overlay CPU plan now records the cumulative number of
`$02:F695` Map32 definitions and whether the quadrant's decompression has
reached its cache-reset statement at each NMI. The translated C decoder has
a resumable tile cursor; it publishes its own decompressed source and Map32
stores through that count. For the first overlay on the route, the plan's
counts are **14, 117, 210, 256**, exactly matching the source's 14, 103,
93, 46 definitions over consecutive hosts. The first count belongs to the
entry host, not the next scheduled NMI. The candidate does not copy source
map values into gameplay.

Binary `a70fd0d2b8cdc8da1c3ddf369f2e8e41c36194e31057a54345f209f5e6f2417a`
matches 56,459 translated-native audio frames
(`target/native-preow-incremental-map-56459`). Its contiguous retained-CPU
trial agrees in all reads, instructions, NMI, PC and clock through host 4,864;
hosts 4,865–4,868 retain the same instruction schedule despite different
partially published Map16-to-Map8 scratch. The next instruction-schedule
split is host 4,869: source `$02:FF0F` wrote the next quadrant's partial
decompression buffer `$7F:4400=$4C` in host 4,868, whereas translated C
still holds that decompression until the screen-build caller returns. The
follow-on work is an interruptible decompressor and copy-to-source phase
driven by CPU write counts, then the same Map32 tile cursor across four
quadrants. A one-byte preservation would leave the rest of the decoder's
live scratch and tile map on the wrong generation.

The same frozen binary also passes the cached **source-route audio** gate over
all 1,581,079 frames, 0 through 1,581,078, with contiguous coverage and no
RNG drift (`target/native-preow-incremental-map-source-full`, manifest and
comparison log). This preserves the established source-ordered audio route;
it does not extend the production-native video frontier or prove the remaining
interrupted decoder state.

### Central CPU schedule for the next overworld callers (working tree)

The next CPU-owner split at host 4,869 belongs to Module08's
`Overworld_LoadAndBuildScreen`, which runs all four screen quadrants. The
ROM shadow now supplies only the per-NMI counts of decompression writes,
source-word copies, and Map32 definitions. The translated decoder publishes
its own scratch and tile writes through those counts, retaining a typed cursor
across NMIs. A ROM-backed regression checks the 15 source crossings and the
staged decoder's final WRAM against the atomic translated decoder. The
contiguous retained-CPU trial moved its first instruction-schedule split from
host 4,869 to host 6,163; the 10,000-frame translated-native audio comparison
is exact (`target/native-screen-build-trial-10000`).

Host 6,163 enters Module09's overlay reload. The same typed Map32 cursor now
advances for this caller from the centrally counted CPU schedule, including
the entry-host definitions and held-NMI progress. This moved the first CPU
instruction-schedule split to host 6,174, again with exact translated-native
audio through 10,000 frames (`target/native-module09-staged-final-trial-10000`
on the retained binary).
At 6,174 the source has returned from Module09 submodule `$21` after writing
`$0710=4`, then accepts an Open NMI. The translated scheduler still holds the
caller until the next host. A trial that returned it before that NMI moved the
CPU split to 6,931, but moved an RNG call one host early near 56,438 in the
longer native audio route. That trial was removed. The next fix must give the
Module09 return and NMI acceptance an owner that preserves both early CPU
ordering and the later sprite/audio ordering; a return branch at this one
continuation is insufficient.

The retained parity binary is
`dba5327bc67bef75dfbf1a1bb3612bb934b8bd2aa45d4ce961b43c9abf1a5e45`.
It matches 56,459 translated-native audio frames
(`target/native-module09-staged-56459`). The same frozen binary passes the
cached source-route audio comparison for all **1,581,079** frames, with
contiguous coverage from 0 through 1,581,078 and no disabled audio lane
(`target/native-module09-staged-source-full`). This is an audio preservation
result; video was deliberately excluded from both checks. The earlier
56,425+ native video frontier and the full source-route A/V proof on prior
binaries are distinct results.

### Module09 body return and NMI handler are separate CPU events

An instruction trace of route hosts 6,162–6,174 shows `$09/$20` enters in
6,162, changes `$11` to `$21` in 6,168 after **six** NMI crossings, and then
enters Sprite_Main before a seventh, caller-owned NMI. The aggregate native
plan needs seven **scheduler callbacks** to reach the matching host phase;
`body_return_nmis` separately records six source CPU crossings. A six-callback
trial put the translated completion at debug host 6,168, which is CPU host
6,167 because `advance_native_exact_cpu_host` labels the retained CPU host as
`frame_ctr_dbg - 1`. The validated seven-callback completion at debug host
6,169 is CPU host 6,168, matching the source `$11` transition. A trial that
also carried the caller's accepted NMI into the next handler host still moved
the native `$1A` increment one host early and changed the thunder cue at
audio frame 6,359. Both trial changes were removed.

The pending NMI command `$0710=4` is written by `$20` in source host 6,168
and stays live through host 6,169. The Open NMI accepted at clock
2,204,897,770 in host 6,169 finishes its handler at clock 2,204,993,868 in
host 6,170; the handler reads and clears `$0710` during that later host. The
retained exact CPU owner now records both accepted and completed NMI events
across hosts (`NativeExactCpuHostTrace::nmi_completions`). Its diagnostic
receipts are `target/native-exact-nmi-accept-6171` and
`target/native-exact-nmi-return-6172`. These CPU events remain separate, but
their separation alone does not justify reducing the aggregate callback count.
The next production change must align the scheduler's entry and return phases
with the retained CPU owner, then place acceptance and handler completion on
their measured hosts. Moving the callback or an individual WRAM command
without that alignment changes later audio timing.

The exact CPU owner also publishes `pending_nmis_at_return` as a typed host
result. At host 6,173 it carries the acceptance at clock 2,206,327,240; at
host 6,174 it records that handler's `RTI` at clock 2,206,423,338 while
carrying the next accepted NMI. See `target/native-module09-pending-nmi-6173.log`
and `target/native-module09-pending-nmi-6174.log`. This makes the pending
phase explicit for a central scheduler handoff; translated gameplay does not
yet consume it.

The final diagnostic binary is
`2d6afad29771ea578f9dd8b2058f86e45513b81aef016527407abca656305829`.
It matches 56,459 translated-native audio frames
(`target/native-module09-pending-nmi-56459`) and all 1,581,079 cached
source-route audio frames, contiguous with no RNG drift
(`target/native-module09-pending-nmi-source-full`). The Zelda library suite
passes 1,813 tests and the SNES library suite passes 496 tests. These checks
exclude video; the Metal adapter failure recorded below still prevents a
native A/V rerun in this process.

The next instruction split is now localized to `$09/$21`. The source writes
`$0710=4` at `$02:EDB1` in CPU host 6,173, accepts another NMI in that host,
and clears the command inside its handler in host 6,174
(`target/native-module09-command-source-6175.log`). The validated native
scheduler completes `FinishWorldMapAmbientMap8` at debug host 6,174 (CPU host
6,173), but also runs the trailing NMI handler before that host returns. Its
frame-6,173 WRAM snapshot has `$11=22` and `$0710=0`
(`target/native-module09-phase-wram-6180`). Thus the trial CPU reads zero at
the start of host 6,174 where the source CPU reads four. This is an
acceptance-versus-handler phase error at the **correct** `$21` body completion,
not evidence for shortening the `$20` plan. A previous trial that moved only
the `$21` return across the handler passed the short CPU schedule gate but
caused an RNG call one host early near 56,438; the handler and resumed caller
must move together under a source-owned CPU phase.

A generic trial that rebased translated WRAM immediately after every carried
source NMI handler returned was rejected. It moved the first CPU instruction
split backward from 6,174 to 3,814 while native audio still matched 7,000
frames (`target/native-exact-cpu-after-carried-nmi-7000`). In the dialogue
initialization crossing, source host 3,812 writes `$0710=2`, host 3,813's
handler clears it, and the translated frame 3,812 still retains the pending
command (`target/native-exact-cpu-phase-3814-source.log`,
`target/native-exact-cpu-phase-3814-native`). The translated host-entry WRAM
can therefore represent either side of a carried handler depending on the
scheduled caller's phase. An `AfterCurrentTrailingNmi` gate did not restore
alignment because the exact CPU owner advances before that host's translated
continuation is staged. The trial code was removed. The scheduler needs an
explicit, persisted host-boundary phase contract before the central owner can
copy translated WRAM at the correct instruction boundary.

After removing the failed six-callback/early-handler trial, the frozen binary
`ac889ebacc61b44c59fdea303a35929c996add912d7a0fdc148bc1bfd448bf15`
matches 56,459 translated-native audio frames
(`target/native-module09-phase-corrected-56459`) and all 1,581,079 cached
source-route audio frames with contiguous coverage and no RNG drift
(`target/native-module09-phase-corrected-source-full`). The library suite has
1,813 passing tests. A native video rerun on this binary could not start: the
offscreen/native-window renderer found no Metal adapter in this process,
although macOS reports the built-in Apple M2 Max GPU and Metal support
(`target/native-module09-phase-corrected-56426-av.log`). Do not treat the
audio gate as a new native A/V frontier.

The frozen binary for this diagnostic increment is
`79646c22b1a6a7156cc9f61a59897318940a3a40a63fed322eefff150c39dc3d`.
Its translated-native audio comparison matches 56,459 frames
(`target/native-exact-nmi-completion-56459`), the library suite passes 1,813
tests, and its cached source-route audio comparison matches all 1,581,079
frames with contiguous coverage and no RNG drift
(`target/native-exact-nmi-completion-source-full`). Video was excluded from
both A/V comparisons. The retained CPU trial's first schedule split remains
host 6,174; this increment improves event ownership evidence, not that
frontier yet.

## Earlier native frontier — 56,425 (video)

`62f43a27`'s inventory-tail continuation moves the native frontier **56,419 -> 56,425**.
The frame-zero baseline reproduces 56,419 in
`target/native-inventory-tail-work/baseline` (74.88s); the candidate is exact
video+audio through 56,424 in `target/native-inventory-tail-work/candidate`
(75.38s). At 56,425 video differs and audio remains exact. Candidate binary
SHA-256: `d13797a118ecd1e593438b01288070af344965da49a55431f14c662ae6c7370b`.
These runs use `ZELDA3_CACHED_AV_NATIVE_TIMING=1`: the ROM timing probe remains
available, but recorded host timing receipts are not installed. This is not
full-route native or ROM-free parity.

The bug class is a missing CPU/NMI continuation. At `$0D:FCEA`, the native
probe reached NMI after the inventory conversions but discarded the boundary
because its classifier recognized only conversions and hearts. The original
tail owns HUD words `$7E:C764` and `$7E:C724`; earlier bow/resource mutations
must remain before the interrupt. `HudInventoryResume::Tail` retains the
computed key digit, backdrop word and completed instruction cycles. The
overworld caller discriminator remains `$02:A4CC`. On resume, only the tail's
remaining stores execute, followed by the existing caller return.

The split is derived from the ROM instructions, not the failing frame:
`REP` 22 + `LDA` 32 + `AND` 24 + `ORA` 24 = **102** master cycles before
`$FCEA STA`. That store ends at 150, `CMP` at 174, and `BNE` at 190 (blank)
or 196 (digit). The blank-label store ends at 238; `SEP` 22 + `RTS` 42
finish at **302** or **260**. The probe counts CPU cycles without bus stalls.
`inventory_tail_instruction_boundaries_match_original_rom_stores_and_cycles`
executes the pinned ROM bytes and checks every instruction edge, all WRAM
writes, both branches and split/resume totals. A second regression changes
keys/arrows across the interruption to detect a repeated inventory mutation.

Targeted inventory tests (17), HUD tests (7), game-state tests (273), and the
production feature check pass. The ownership scan has the same existing
diagnostics as the baseline and no new overlaps.

Full-route preservation passes on the same binary: **1,581,079 exact A/V
frames**, from frame zero with no paired resume, in 1625.82s
(`target/native-inventory-tail-full`). There is no RNG drift. All four
WRAM goldens match, as does the complete final WRAM image, SHA-256
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
This full run installs the established timing receipts; it preserves that
lane's parity while the native frontier remains 56,425. The pinned cold check
passes 57,000 video frames and 30,387,879 stereo sample frames exactly
(`routes/full_run/comparisons/precommit/run-57000-exact-eo5czasy`, 448.0s),
after live-oracle RNG calibration and video preflight. Its immutable receipt is
`.git/parity-cold-passes/1789344555828402000-57000-37ad9e5ebaff.json`.
The local cold ratchet remains 194,000; this candidate's focused cold proof is
57,000, and its full-route proof is the cached A/V run described above.

The independent frame-zero confirmation in
`target/native-inventory-tail-work/confirm` reproduces 56,425 (77.31s).
`ZELDA3_DEBUG_OVERWORLD_CPU_ITERATION=56418-56428` confirms the now-owned
`$0D:FCEA` boundary at host 56,419 V225/C14, then hearts `$0D:FDB8` at
56,421 and decimal conversion `$0D:F124` at 56,423. The next boundary is
host 56,425 **`$06:F80F`, V225/C28, `link_oam_caller=None`**. This is outside
the LinkOam/HUD suffix; diagnose its actual caller and owned mutations before
introducing another continuation. WRAM 56,418..56,428 is retained beside the
confirmation's A/V ledgers. Logs and test output are copied to
`target/native-inventory-tail-work/validation`.

### Next-boundary investigation (2026-09-23)

The source receipt for host 56,424 accepts NMI after Cucco slot 6's first
graphics publication; host 56,425 returns from `Sprite_Main`. The native ROM
probe at `frame_ctr_dbg=56425` reaches `$06:F80F` (V225/C28), inside
`Sprite_SetupHitBox` for that same slot. Its `Cucco_AnimateFast` graphics
store at `$06:A6F7` wrote `$0DC6` before the interrupt. This helper can be
reached through a tail jump, so the top stack return (`$00:83A6` here) does
not identify the Cucco call site. A detector keyed only to the hitbox PC or
to a JSR return address is insufficient.

An uncommitted experiment used the actual Cucco graphics store to arm the
existing `AfterCuccoGraphicsPublication` continuation. It changed the native
frame-56,425 video digest from `d05febca...` to `0f28586e...`, but the oracle
is `e7536701...`; audio remained exact. The experiment was removed. At that
frame, native and receipt-driven WRAM differed only at transient `$1F00`.
Their captured PPU states had identical OAM and CGRAM, but 337 different VRAM
words in `$3C00..$3DFF`, different OBJ decode latches, a receipt-only BG
decode latch, and BG1/BG2 scroll values one pixel ahead in the receipt lane.
The unresolved owner is the NMI/display publication and resumed Module09
suffix around this interrupt, not a proven Cucco gameplay-state mutation.
Trace those publication phases before introducing a native continuation.

The next probe isolated the ordering error. The native timing shadow reaches
the Cucco interruption with scroll source words `$00E6=08DF`, `$00E2=00B0`,
and `$00E8=08D8`, while the translated mirrors still hold `08DE`, `00AF`,
and `08D7` when its NMI writes PPU registers. The source acceptance operand
at host 56,424 also contains the newer values. The translated Module09 camera
and scroll-prefix work runs later in native host 56,425; a diagnostic call
stack shows its earlier NMI came from `lane_rom_startup_run_main`'s active
scanout path. The source host 56,424 contains both completion of the preceding
Open NMI and a later Held acceptance inside Cucco. The existing lane presents
the leading NMI and suspends the fresh iteration without owning that later
acceptance in the same host interval. This is a two-NMI/scanout ownership
problem; merely arming `AfterCuccoGraphicsPublication` retains the CPU stack
but still presents the old scroll and animated BG generation. Correct the
measured phase and display publication in that lane before using the Cucco
checkpoint as a native continuation. The experiment was removed; main and
the 1,581,079-frame receipt proof are unchanged.

### Native Cucco acceptance correction (working branch)

The measured `$06:A6F7` graphics store now identifies
`AfterCuccoGraphicsPublication` without guessing from the later hitbox PC.
The Module09 leading-NMI lane retains its current scanout, captures the
post-camera acceptance state, and completes that Held NMI against the
captured state on the following host before `Sprite_Main` returns. Its
animated BG generation comes from the already completed leading NMI. This
keeps both NMI/display generations in their source order; copying timing
probe RAM into native state is unnecessary.

The cleaned native candidate matches cached video and audio through frame
56,438 inclusive (`target/native-cucco-clean-56442`); the next video-only
divergence is frame 56,439. The source receipt at host 56,438 interrupts
`Sprite_Main` after timers/OAM in slot 3, while the native CPU probe at
`frame_ctr_dbg=56439` reaches `$00:878E` at V225/C34. The branch's frozen `target/parity/zelda3` binary also
passed the full cached receipt route in `target/receipt-cucco-full`: all
1,581,079 video and audio hashes match, with contiguous frames 0–1,581,078
and no disabled lanes. The remaining promotion gates have not run for this
batch.

### Native post-timer Sprite_Main dispatch (working branch)

An instrumented Snes9x trace (`target/source-f56439-pc-trace`, run 56,438)
shows that the source also accepts the held NMI at `$00:878E` (V225/C16).
The earlier interpretation that native CPU execution was several sprite
slots ahead was wrong: both machines entered slot 3 and returned from
`Sprite_TimersAndOam` at `$06:84EB`. The source's
`SpriteMainAfterTimersAndOam(3)` receipt describes the still-active call
stack at `JumpTableLocal`, not a different interrupt PC.

The native timing probe now recognizes that post-timer checkpoint only
when `$00:878E` retains Sprite_ExecuteSingle's `$00:83A6` return frame
and the current slot matches the observed timer return. This leaves later
handler checkpoints with their own owner: a broad timer-return rule
incorrectly claimed item-receipt graphics at host 47,125 and was removed.
The Module09 held-NMI display snapshot path applies to this typed
Sprite_Main continuation as well as the preceding Cucco case. The cleaned
native comparison matches frames 56,437–56,457; the next video-only
divergence is frame 56,458 (`target/native-after-timers-clean`). The same
cleaned binary matched receipt-driven video and audio for all 23 frames
56,437–56,459 (`target/receipt-after-timers-clean`). The previous full
1,581,079-frame receipt proof belongs to the preceding Cucco commit; this
increment has not repeated the full route or promotion gates.

### Interruption-class audit before the next native change

`scripts/sprite_main_interruption_matrix.py` streams the immutable timing
receipts and groups `SpriteMainProgressed` by the NMI acceptances in its host
and the following host. Through host 56,459, six post-timer progress hosts have
no NMI acceptance of their own and are followed by a held acceptance; five
return `Sprite_Main` on that following host (24,651, 27,137, 30,771, 31,283,
56,457), while the item-receipt caller at 20,257 remains suspended. The same
matrix preserves multiple acceptances within one host, as at 56,438. This
rules out treating a post-timer progress receipt as synonymous with an
accepted NMI or a ready-to-return Sprite_Main stack.

At the new native video frontier, source run 56,458 accepts its held NMI at
V225/C20, PC `$06:DCE3` inside `Sprite_DrawShadowEx_`, with slot 2 active.
The clean native CPU probe for the corresponding iteration reaches its NMI
boundary at V225/C12, PC `$06:A618` inside `Sprite_0B_Cucco`; audio is still
exact and video first differs at frame 56,458. The source trace is in
`target/source-f56458-pc-trace`; the native comparison is in
`target/native-systemic-baseline`. The receipt names this call stack
`AfterTimersAndOam(2)`, but that checkpoint does not describe the partial
Cucco/shadow draw at acceptance. Restore the source-ordered CPU work first,
then re-evaluate whether an instruction-boundary draw prefix and resumed
stack remain necessary. Widening the existing post-timer rule would erase
the distinction between this case and the item-receipt continuation at host
20,257 (and the previously observed item decompressor at 47,125).

The immediate display difference is the captured BG scroll, not OAM: source
scanline zero uses `(259,2271)/(210,2265)`, while native uses
`(258,2271)/(208,2265)` with `CapturedBeforeNmi`; displayed OAM entries
12–21 match. The CPU timing gap is upstream of the partial draw. Source and
native agree within 22 master cycles at Cucco's `$06:A618`, but differ by
326 cycles after the avenger's `Sprite_ApplySpeedTowardsLink` call. The
32-iteration projection loop takes different branches with the same X
sequence; both sides charge the same 1,344 HDMA master cycles over its 32
scanlines. The first differing input is the avenger's `GetRandomNumber`
result at `$0D:BA71`: source seed `$C3` yields `$8C`, while the isolated
native ROM timing shadow yields `$55`. At source V83/C762, `$2137` latches
H=190; the subsequent `$213C` read at C792 returns `$EA` because the source
PPU is on its high-byte read phase and preserves the PPU bus bits. The native
shadow starts this frame with a fresh counter-read phase and returns a low H
byte instead. Source and native WRAM match at the pre-frame boundary,
including seed `$C3`, Link coordinates, and sprite slots. This is a PPU
read-register continuation ownership problem, not a Cucco-specific cycle
constant or a translated spawn-coordinate defect. The source proof is in
`target/source-f56458-rng-ppu-trace` and
`target/source-f56458-ppu-read-trace`; native exact-PC probes use
`ZELDA3_DEBUG_OVERWORLD_CPU_ITERATION=56458-56458` with
`ZELDA3_DEBUG_OVERWORLD_CPU_PCS=06:a7f5,06:a7f9,06:e9c8`.

The source read-state lineage is explicit in the paired-resume trace. Resumed
run 5 reads `$213C` low as `$0E`; run 10 first reads high as `$0E`, then
reads low as `$EB` at V146/C972. No `$213C` or `$213F` read occurs from
run 11 through run 28. Run 29's high read preserves `$EB`'s upper seven bits
and replaces bit zero with the newly latched H counter's high bit, producing
`$EA`. This state crosses 19 source runs; a frame-56,458 seed or a local
Cucco branch adjustment cannot derive it.

An experimental copy of PPU counter-read state from each overworld timing
shadow into the translated PPU was rejected: it shifted receipt RNG call
order at host 55,074, before the present frontier. A correct fix needs one
source-ordered owner for `$2137/$213C/$213F`, including read flip and PPU
open-bus values, across all CPU timing plans and frame boundaries. The
existing `SourcePpuReadState` has the right register semantics, but the
native `RomCpuTimingRun` still executes instruction side effects before
charging aggregate cycles and rebuilds its `Snes` shadow for each plan.
The next clean slice is to make that source-ordered read state a retained
native CPU peripheral, seeded from reset and updated at the actual bus
access timestamp. The central `CpuSynchronousMachine` already owns the
source-ordered NMI/IRQ, HDMA, APU, and nested nonzero `$420B` general-DMA
drain; the separate `RomCpuTimingProbe` still rejects nonzero `$420B`.
Native integration should retain one central owner across plans rather than
copy its register state into fresh legacy shadows or duplicate DMA in the
probe. Translate each caller's semantic writes at its actual source bus
boundary, and require the central machine's CPU/PPU/APU cursor to agree
before a plan returns. Compare the complete `$213C`
read sequence above and the first RNG/call-order deviation before measuring
the new A/V frontier. Do not promote the isolated overworld-only copy or
compensate with an extra 304 cycles. The diagnostic experiment was removed
from the branch.

An exact resumed source checkpoint now witnesses the handoff directly:
host 56,457 reads `$2137` at master clock 20,175,926,238 and `$213C` at
20,175,926,268, obtaining `$EA`. Its completed host retains
`h_read_high=false`, `open_bus2=$EA`, and latched H=190. Host 56,458 begins
with that same PPU read owner; NMI is accepted during opcode `$98` at
`$06:DCE2` at clock 20,176,119,184. The opt-in
`ZELDA3_SOURCE_TRACE_PPU_READS` and `ZELDA3_SOURCE_TRACE_INTERRUPTS`
diagnostics produce this witness from the same exact checkpoint without
changing the normal execution path. It proves source ordering and owner
retention, not translated-native video parity.

The committed `455066b1` parity binary (SHA-256
`4653467c159380847d44e9a4051359acb1d3e0a8d265f22c4a4c67a830c2a684`)
matched both cached A/V lanes for all 1,581,079 contiguous source-route
frames. Its translated native audio-only lane matched 56,482 contiguous
frames (hosts 0 through 56,481). The next host panicked under strict RNG
replay: `cucco_summon_avenger` called `GetRandomNumber` at execution frame
56,482 while the next source sample belonged to 56,488. This is a
downstream call-order witness on the same avenger path, not a new video
frontier. The native video lane could not run in that session because wgpu
found no Metal adapter, despite macOS listing the built-in M2 Max GPU.

The first central-owner integration seam is now in `CpuSynchronousMachine`:
`write_ppu_register_alias` commits a translated PPU register write and its
source bus access on the machine's current timeline. Exact 65816 execution
uses the same PPU-write semantic, and a synthetic cold-CPU transaction test
checks the external path's PPU result, OpenBus retention, and end timestamp.
`read_ppu_register_alias` likewise shares the exact CPU's audited PPU read
owner, including OpenBus1 producers (`$2134..$2136`, `$2138`, `$213E`) and
counter ports (`$2137`, `$213C`, `$213D`, `$213F`, `$4213`). A cold-CPU
transaction test proves the `$EA` high read, and a failed-drain test proves
that resumption returns the retained byte without flipping the read phase
again. `write_wrio_alias` shares the exact CPU's `$4201` semantic; a V83/C760
high-to-low WRIO edge test captures H=190 and V=83 before its bus charge.
`publish_cpu_open_bus` is the matching post-drain publication step. It refuses
to publish while a bus transaction has a pending completion; the exact cold
CPU's instruction bus uses the same guard, and tests cover normal and failed
PPU reads. This makes the transaction boundary explicit, but does not supply
the production translated caller's retained CPU cursor.
The owner also accepts exact-timed direct and mirrored WRAM reads/writes through
`read_wram_alias` and `write_wram_alias`. The cold CPU and external caller share
one address resolver, and a two-access test compares memory, bus timestamps,
and OpenBus publication. A failed-drain read retains its original sampled byte
even if the WRAM cell changes before resumption. Other bus addresses are
rejected rather than treated as ordinary memory.
`fetch_pcbase_opcode_alias` and `advance_cpu_add_cycles_alias` now expose the
two distinct instruction-clock operations from the same owner. The exact
cold CPU uses those methods for its opcode fetch and `AddCycles` transaction;
an external NOP test checks opcode, PC, transaction timestamps, and OpenBus
against a cold step. The fetch intentionally leaves due events pending while
`AddCycles` drains them. Neither method accepts a fresh raster estimate, and
both refuse to overtake an unfinished bus completion.
`AddCycles` now retains a typed completion if its event drain fails. A test
places a PCBase fetch across HMax, forces the following APU drain to fail,
then resumes it and matches a successful one-shot reference without a second
opcode fetch or cycle charge. The cold CPU still poisons a failed partial
instruction and drops that external continuation; only a translated caller
which retains its instruction state may resume it.
The branch parity binary SHA-256 `2a6a006aca3791031ea15e52ca0d65affd8a705054dd962728094ebe2151150b`
matched all 1,581,079 contiguous source-route audio frames in
`target/native-timing-owner-full-audio`. The translated native lane matched
56,482 contiguous audio frames in `target/native-timing-owner-native-audio-prefix`
and still reaches the strict RNG call-order failure at host 56,482. The
one-frame video smoke attempt still failed to acquire a Metal adapter, so
neither a branch full A/V gate nor a new native video frontier is established.
No native plan calls this API yet; the next integration must supply its actual
bus boundary and retain the CPU cursor instead of reseeding a fresh timing
shadow from the translated PPU image.
An opt-in experiment retained the entire legacy `RomCpuTimingRun` across the
overworld main-wait boundary. It matched 10,000 native audio frames, but its
first WRAM difference from the translated owner appeared at host 2,337
(`$0000`, shadow `$7D`, translated `$A8`). At host 41,225, the retained run's
module-dependent stop PC was `$00:805D` while the translated route entered
Module0F at `$02:9982`; the cross-module plan rejected the mismatch. Evidence:
`target/native-retained-shadow-10k` and
`target/native-retained-shadow-56500.log`. The experiment was removed. A
retained legacy shadow cannot be promoted as the native owner without ordered
reconciliation of translated writes; silently changing its stop PC or
rebuilding it at that transition would hide the ownership gap.
The exact cold source CPU's return probe confirms that WRAM `$10` remains
`$09` at hosts 41,223–41,227, with no `$10` write in that interval, while the
translated route has entered its Module0F path at host 41,225. The source
trace is `target/native-retained-shadow-source-module-value.log` (the probe
now prints watched WRAM at each traced host return). These are distinct
internal representations at a common host boundary, so whole-WRAM equality
is not a valid prerequisite for a persistent source CPU timing owner.
The first tempting caller, `native_main_loop_cpu_run`, has no such boundary:
it reconstructs `RomCpuTimingRun` from translated WRAM/PPU/DMA and advances a
separate aggregate `CpuCycleBudget` after each legacy instruction. The legacy
`PpuState::read($213C)` high phase returns only the counter's bit zero, while
the source owner returns `(OpenBus2 & $FE) | bit0`; retaining the legacy flip
would still not produce `$EA`. A native integration needs a persistent
source-ordered CPU/bus cursor across the main wait, NMI, and module plans,
with translated writes reconciled at their actual bus transactions. The
external PPU-write seam is tested for post-semantic drain failure and resume
without replaying a scroll write, but must not be called from a synthesized
frame raster position.

The source-ordered timing probe now executes active HDMA init/scanline
events at their CPU timeline deadlines and charges the DMA core's measured
cost plus Snes9x's two sync clocks. A source-shaped channel-7 mode-2 test
checks the 42-clock scanline stall and descriptor advancement. The probe
also schedules an enabled VBlank NMI for H=12 while leaving interrupt entry with
the external owner; boundary tests check that handoff and retention of the
`$213C` read flip/open bus across field rollover. These are prerequisites,
not a native-parity result: `RomCpuTimingRun` is still the aggregate timing
owner, and the source-ordered probe still needs audited MMIO coverage and
a persistent native checkpoint before an overworld plan can use it.

The probe can now transfer an opaque instruction-boundary handoff containing
the CPU machine, synchronous timeline cursor, PPU read state, and adopted APU
owner. A two-plan test reads `$213C` low as `$EB`, crosses field rollover,
then reads its retained high phase as `$EA` after handoff. ROM writes to HDMA
channel registers and `$420C` now configure the same channel owner that the
timeline later charges; an unowned dynamic HDMA request fails before mutation.
Auto-joypad result reads, arithmetic results, and NMI-enable changes outside
the pending VBlank edge also have source-ordered bus semantics. The handoff is
not yet stored in `ZeldaState`: its current `CpuCycleBudget` owns a separate
legacy timeline, and the production `RomCpuTimingRun` still executes aggregate
instructions. Both must be replaced as one owner before using this path for
native timing or changing the parity frontier.

`CpuCycleBudget` can now observe a source instruction or NMI receipt by
adopting the probe's synchronous timeline cursor. It checks that the entry
clock, exit clock, bus workload, and field schedule agree, then reports the
same typed boundary without charging CPU work, refresh, or HDMA a second time.
Focused tests cover a refresh-crossing instruction, NMI entry after a boundary,
and a mismatched odd-field schedule. This is an integration seam, not a native
route switch: `ZeldaState` still runs `RomCpuTimingRun`, so no A/V frontier
change is claimed. The next step is a persistent source CPU/peripheral owner
across native plans, including poly-thread IRQ scheduling, before replacing
the aggregate run and budget together.

The source probe now carries `CPU.NMIPending` and its H=12 acceptance deadline
through the opaque handoff. It rejects early external entry and refuses the
next CPU instruction once the deadline is due until the interrupt owner enters
the NMI. The boundary test crosses VBlank, hands off at H=14, verifies the
blocked instruction, then executes the native entry; an ambiguous mid-VBlank
seed also fails closed. A separate read witness confirms that acknowledging
RDNMI (`$4210`) does not discard the independently scheduled CPU NMI. This
removes a silent timing divergence in the probe.
Automatic interrupt dispatch, poly-thread IRQ, and a persistent native-route
owner remain separate work.

The source probe now routes `$2100..$2133` writes to the same PPU register
owner as the exact cold executor, at each ordered CPU bus access. A two-plan
test retains INIDISP and OBSEL writes across an opaque handoff; the external
ROM cold-boot witness passes the former `$00:8018`/`$2100` stop while retaining
its recorded first four APU write timestamps. This removes one unsupported
register class from the probe. It does not yet supply automatic interrupt
dispatch, general DMA, or a persistent native route owner.

The plan probe now schedules vertical-only IRQs from source-ordered `$4200`
and `$4209/$420A` writes using the same pinned deadline calculation as the
cold CPU owner. It retains the timer and a selected IRQ through an opaque
handoff, blocks the next opcode until the external owner enters IRQ, and keeps
the IRQ line asserted until `$4211` acknowledges it. CLI/SEI select using the
previous I flag while the pushed status uses the new flag; a simultaneous
VBlank NMI takes priority. Focused probe tests cover these boundaries and
the full SNES library suite passes. Horizontal IRQ, automatic native-route
interrupt dispatch, general DMA, and the persistent `ZeldaState` owner remain
unimplemented. This is a probe capability, not a production A/V frontier move.

The retained probe now exposes one `advance` entry point that selects a due
NMI or IRQ before fetching another opcode and returns a distinct instruction,
NMI, or IRQ receipt. Its tests cross a plan handoff with a pending interrupt
and check simultaneous VBlank NMI priority over vertical IRQ. This removes
interrupt selection from future native plan callers; the separate low-level
`step` and interrupt-entry methods remain for timing tests. General DMA and
the persistent `ZeldaState` source CPU owner are still required before the
aggregate native timing path can be replaced. This API alone does not change
the production native A/V frontier.

The central cold CPU owner completed the 1,581,079-host recorded route with
exact presented OAM at every host. A separately resumed final checkpoint at
host 1,581,078 matched the oracle's CPU registers, all 128 KiB of WRAM, and
the cartridge's 8 KiB SRAM with zero differences
(`target/source-cli-fixed-final-state-comparison.log`). This validates the
source CPU owner over the full route, not the production translated native
renderer or its frame-56,458 video frontier. The production parity binary
remained SHA-256 `a790c35bbd13294ea370ab3d7d4bd37dc37a65f76d2af26ee04c62aba055c4fe`;
the next cold gate matched 204,000 exact A/V frames from frame zero.

The native budget's source-observation seam now consumes the probe's typed
instruction/NMI/IRQ advances with one timeline. In this mode, NMI acceptance
is reported only after the source CPU actually enters NMI; crossing nominal
H=12 with NMI disabled no longer fabricates a boundary. A real vertical IRQ
test retains the CPU and clock across plan handoff. The production path still
needs a persistent source CPU owner in `ZeldaState`, ordered reconciliation
with translated memory writes, and source-owned DMA/APU/display publication
before the aggregate timing runner and synthetic poly-thread clock can go.

The exact reset-proven CPU can now transfer its quiescent CPU, timeline, PPU
read latches, APU coroutine, and unpublished DSP samples into the probe. A
frame-56,458 source checkpoint kept CPU registers, WRAM, and master clock in
agreement for 14,522 instructions before the probe reached its unsupported
nonzero `$420B` general DMA path. The exact CPU already executes that DMA.
Its instruction receipt now names accepted NMI/IRQ entry and its clock, and
`CpuCycleBudget` can observe that same source machine through a nested DMA
without charging the transfer again. This is a development ownership seam;
production translated native video still first diverges at frame 56,458.
At the saved source checkpoint immediately before that host, `$06:DCE2`
starts at master clock 20176119170, accepts NMI at 20176119184, and finishes
its entry at 20176119246. The event is selected after the opcode, so its
acceptance clock cannot be inferred from nominal V225/H12 alone.
The parity binary SHA-256 `ee5e0fd6338fda18aceb54d984e8524a8e59f6f2a57cfb9605f0c086997dd71e`
matched all 1,581,079 contiguous cached video and audio frames in
`target/exact-source-budget-full`. The same binary's production native timing
path still first diverged at frame 56,458, video only, in
`target/exact-source-budget-native`.

Promotion tooling also now accepts an explicit `--frames` limit exactly equal
to the cache's full frame count. The previous validator rejected every explicit
limit, including this complete 1,581,079-frame run. The correction preserves
the original manifest and still rejects partial coverage, resumed runs,
mismatches, disabled lanes, and malformed or out-of-range bounds. Regression
tests reproduce the rejection and check both the accepted endpoint and those
failure cases; the evidence and pre-commit gate Python suites pass.
The full-route pass is promoted for `8d3c640a` in
`routes/full_run/parity-frontier.json`, with the unchanged replay manifest
retained at `routes/full_run/receipts/native-inventory-tail-full.manifest.json`.

## Previous native frontier — 56,419 (video)

`fa1972e3` owns the second ordinary-overworld interruption class, moving the
frontier 56,417 -> 56,419 (audio exact at both). Binary in
`target/native-hearts-fix` (70.52s): exact video+audio 0..56,418.

The mechanism is `cba205c8`'s, one routine further along the suffix. A captured
PC trace showed the iteration started at frame 56,416 V255/C224 was still
running at the NMI of 56,417 (V225/C34), so its handler ended at V227 instead
of V250 and the interrupted iteration resumed and returned at V234/C946 with no
new iteration in that host. The native measurement already reached that
boundary at V225/C20, PC `$0D:FDB0` inside `Hud_UpdateHearts`, and dropped it.

`HudUpdateInterruption::InsideHearts` resumes through the existing
`HudUpdateResume::BeforeHearts` callee re-run. **That is exact for the hearts
routine specifically, and the reason does not generalise:**

- `Hud_UpdateHearts` ($0D:FDAB..$0D:FDD8) plus `Hud_UpdateHearts_DrawHeart`
  ($0D:FDD9..$0D:FDEE) store only `$00`, `$07` and `[$07],Y` — their own loop
  count, their own row pointer, and HUD tile words. No game state.
- The buffer is published only when `$16` is set (`$00:8B67 LDA $16 : BEQ`),
  and `$16` rises only at `$0D:DD26 INC $16`, after the callee returns.
  Measured on the trace: 56,416 raises `$16` at V183 and the gate uploads at
  V242; 56,417's interrupting NMI has no `$16` write before it and never
  reaches `$00:8B67`; `$16` rises at V234 after the resume, and 56,418's NMI
  uploads it. The partial buffer is never seen.
- The re-run derives its count from unmutated health capacity/current health,
  so the source's partial pass is a subset of the same words.

### Original diagnosis — 56,419: the inventory tail (resolved above)

Native's boundary is `$0D:FCEA` at V225/C14; the source's NMI for that frame is
V225/C12. `$0D:FDB8`, `$0D:F124`, `$0D:F105` and `$06:F80F` follow close behind.
`$06:F80F` is outside the suffix's LinkOam/HUD chain entirely
(`link_oam_caller=None`) and needs its own owner.

**Do not extend `InsideHearts` to cover the inventory.**
`hud_update_inventory_from` writes game state — `set_inventory_item(0, …)` for
the bow slot — not only HUD tile words, so re-running the block whole would
defer a real WRAM write by a host. That is why the inventory path already
carries `HudInventoryInterruption { field, entry_master_cycles, master_cycles }`
and `HudInventoryResume`.

But the existing grammar does not fit either: it models "suspended *inside* a
field's decimal conversion", consumed in `hud_inventory_decimal` before any of
that field's digits are written. `$0D:FCEA` is **after** all four conversions,
in the block's tail. Forcing it into `field: Keys` would mean synthesizing
`entry_master_cycles`/`master_cycles` for a conversion that actually completed
— a guessed offset.

The tail itself is well behaved. `$0D:FCE0-FCF9` is
`REP #$30 : LDA $05 : AND #$00FF : ORA #$2400 : STA $7EC764 : CMP #$247F :
BNE : STA $7EC724 : SEP #$30 : RTS` — `$7EC764` is `hudxy(18,1)` and `$7EC724`
is `hudxy(18,0)`, both HUD tile words, derived from the Keys digit and
`rupees_actual()`. No game state. So the needed shape is:

1. `HudUpdateInterruption::InventoryTail`, detected for a boundary in
   `$0D:FCE0..$0D:FCF9` with the same `$02:A4CC` `Hud_RefillLogic` caller
   discriminator `InsideHearts` uses.
2. `hud_update_inventory_from` returning a third outcome rather than
   `Option<HudInventoryResume>` — e.g. `enum { Completed, Conversion(..), Tail }`
   — with the tail extracted into its own method taking the computed `key`
   word, which is exactly the `$05`-derived value the source has live at the
   boundary.
3. `HudUpdateResume::InventoryTail { key }` running only that method.

**The open sub-problem is the cycle split.** The tail is currently charged as
one `190 + (48|6) + 64` block for `$fce0-fcf9`. Suspending at `$FCEA` has to
split that 190 at the `STA $7EC764`, which means pricing
`REP #$30 : LDA $05 : AND #$00FF : ORA #$2400` from their own access costs and
proving the two halves still sum to 190. Do that with
`docs/parity/cycle-ledger-recipe.md` before writing the continuation; charging
the whole 190 on the resume (or all of it before) would be a guessed offset of
exactly the kind this program refuses. This is why the fix was not attempted
alongside `fa1972e3`.

## Previous native frontier — 56,417 (video)

`cba205c8` gave the native lane its first main-loop interruption owner for the
ordinary overworld, moving the frontier 56,390 -> 56,417 (audio was already
exact at both). Binary SHA
`9f4938f6d9fdf97ebefa363e2785bce18d1cf7c56dedd61674c0e27a605dea3b`,
`target/native-linkoam-confirm` (70.76s): exact video+audio 0..56,416.

### The method that found it (reuse this — it is a ~2 minute loop)

1. Dump both lanes' presented state at the failing engine host and diff it:
   `ZELDA3_DEBUG_PRESENTED_FRAMES=<frame+1> ZELDA3_DEBUG_PRESENTED_DIR=<dir>`
   once with `ZELDA3_CACHED_AV_NATIVE_TIMING=1` and once without. At 56,390 BG
   VRAM and CGRAM were identical while the snapshot RAM differed in 109 bytes
   that were all one step apart — frame counter `$ae` vs `$ad`, Link, camera,
   BG scroll. That is "native ran an iteration the source held", not a
   rendering or beam-counter bug, and it takes two 70s runs to establish.
2. Read the source's disposition for those hosts with
   `ZELDA3_DEBUG_INSTALL_RECEIPTS=<lo>-<hi>` on the receipt lane
   (`--ignore-video`, ~43s). Host 56,389 was
   `MainLoopInterrupted(LinkOam)` and 56,390 `CallStackContinued`.
3. Read what the native ROM-CPU measurement saw for the same hosts with
   `ZELDA3_DEBUG_OVERWORLD_CPU_PACKING=1` on the native lane (~52s). It had
   already reached the NMI boundary at `$0D:AAB9` and thrown it away.

Note the two lanes' host labels can differ by one: the packing trace prints
`state.frame_ctr_dbg`, which a trailing-NMI host leaves one below the
receipt's `host_call`.

### Next native frontier — 56,417: an unclassified NMI boundary in the HUD

**Settled by a captured Snes9x PC trace** (`00:8051,00:8056,00:805a,00:805d`,
whole 56,421-frame prefix). Two earlier readings of this frontier were wrong
and are retracted: it is neither a host return before the common suffix, nor a
cycle-cost difference. It is the same shape as 56,389 — the native measurement
reaches the NMI boundary and the classifier does not recognise the PC.

The source's per-frame main-loop landmarks around the frontier:

| frame | `00805a` return | `00805d` | NMI | handler end `008225` | `008051` | `008056` |
| --- | --- | --- | --- | --- | --- | --- |
|56,414 | V189/C610 | V198/C722 | V225/C28 | V250/C1160 | V251/C118 | V255/C206 |
|56,415 | V213/C314 | V222/C426 | V225/C20 | V250/C1148 | V251/C128 | V255/C216 |
|56,416 | V183/C822 | V192/C934 | V225/C24 | V250/C1156 | V251/C136 | V255/C224 |
|56,417 | V234/C946 | V243/C680 | V225/C34 | V227/C124 | — | — |

An iteration starts at `008056` near V255 and returns at `00805a` in the
*following* frame. Frames 56,414-56,416 return early (V183-V213), so the NMI at
V225 finds the main loop idle and a fresh iteration starts at V251/V255. Frame
56,417 is different: the iteration started at 56,416 V255/C224 was **still
running** when the NMI arrived at V225/C34, so the handler ends at V227 instead
of V250, the interrupted iteration resumes and returns at V234/C946, and no new
iteration starts in that host. That is exactly host 56,416's receipt
(`IterationStarted, SpriteMainReturned`, no suffix) followed by 56,417's
(`NmiAccepted(LatchHeld) … CallStackContinued, MainLoopCommonSuffixCompleted`).

The native measurement already sees it. `ZELDA3_DEBUG_OVERWORLD_CPU_ITERATION`
reports for that iteration: entry V251/C38, `008056` at V255/C202, and an **NMI
boundary at V225/C20 with PC `$0D:FDB0`**. The source's NMI is at V225/C34, 14
master cycles later. Native's returns for the neighbouring iterations match the
source to 0, +8 and −30 master cycles (`00805a` V189/C610 vs V189/C610,
V213/C322 vs V213/C314, V183/C792 vs V183/C822) — the measured iteration cost
is right; the earlier "~40 scanlines of missing work" was a misreading of the
receipt vector, not a measurement.

`$0D:FDB0` is `CMP #$0008` in the HUD's heart/inventory drawing loop
(`$0D:FDAE LDA $00 : CMP #$0008 : BCC : SBC #$0008 : STA $00 : LDY #$0004 :
JSR $FDD9`). The neighbouring unclassified boundaries are `$0D:FDB8` and
`$0D:FCEE`. The existing classifier knows only `$0D:FB94` (before hearts),
`$0D:FC57/FC84/FCA8/FCDD` (inventory fields) and `$0D:F0F7..F127` (conversion),
so these fall through to "no interruption" exactly as `$0D:AAB9` did.

So the fix has the same shape as `cba205c8`, but the continuation is the hard
part: unlike `LinkOam_Main`, `Hud_Update` publishes as it goes, so resuming it
whole is not obviously safe. Establish what `$0D:FDD9` and its caller own
before choosing between a finer `HudUpdateResume` variant and a
progress-carrying continuation. Use the trace: it is cheap to re-capture and
gives the source's own interrupted PC and resume point.

## Previous native frontier — 56,390 (video): dialogue/return timing

The native dialogue/return timing batch restores exact A/V through56,389;
56,390 still fails video with audio exact. These are source timing and
CPU-state ownership fixes, not offsets chosen to move the frontier:

- `RenderText_Draw_Finish` now charges the original `$0e:ca35..ca6b`
  straight-line738 clocks. Its separately annotated border initializer
  costs204. The regression asserts942 total clocks, the exact upload bytes,
  module return, and absence of writes outside the finish block's ownership.
- Fresh dialogue iterations use the retained main-wait checkpoint through
  the leading NMI. When the iteration reaches the common return, the probe
  retains its actual next CPU checkpoint/budget instead of reseeding the
  next caller. Interrupted glyph/scroll work remains with its translated
  continuation; the probe does not predict future joypad input or copy
  shadow RAM into gameplay.
- A measured glyph entry retains its physical field as well as its raster.
  Its budget uses the actual alternating field lengths, including the
  four-clock short odd scanline240. Resumed dialogue uses the same physical
  field convention. A regression covers both field lengths and stall costs.
- The song-upload sprite-preparation suffix retains its CPU return phase
  after its accepted NMI and typed continuation. Source comparison54,252
  and native effective host54,253 both reach `$8034`, V225/C28, counter173:
  `target/native-dialogue-upload-phase/upload-return-phase-comparison.json`.
  This proves that return checkpoint, not every subsequent module entry.

`target/native-dialogue-upload-phase` is the frame-zero native run (81.55s).
Binary SHA:
`1db17ef5c24a22ebd279ae3bb3784c1e79e8d857659bba1a75d3443e563c1538`.
All1,798 library tests pass,3 ignored (23.82s):
`/tmp/native-dialogue-upload-phase-lib-tests.log`.
`target/native-dialogue-upload-phase-receipt` matches all56,397 receipt-driven
frames (63.85s).
No full1,581,079-frame run was repeated for this binary. The older full
receipt result remains evidence for its recorded binary only.

An intermediate run, `target/native-dialogue-main-wait-field`, has the same
native frontier; its `dialogue-return-comparison.json` confirms source/native
module, counter, and all OAM bytes agree across52,748..52,756. The dungeon
command still reachesV118/C220 versus sourceC234,14 clocks early. Restored
audio parity is not proof of exact absolute CPU phase. Do not compensate
that residual with a command offset.

### APUI ownership and the shared beam-counter owner (this batch)

Two hardware owners the development executors lacked. Neither is reachable
from the native route yet — nothing outside the snes crate's own tests
constructs `RomCpuTimingProbe`, `ApuHostPortProbe`, `ApuHostPortTiming`,
`Snes9xColdCpuExecutor` or `CpuSynchronousMachine` — so the native frontier is
unchanged and no A/V claim is made for either.

`ApuHostPortTiming` (`3f44007a`) is the APUI owner the source NMI prefix
stopped at. It pairs `ApuHostPortProbe`'s retained SPC continuation with
`Snes9xApuClockState`, so pinned `S9xAPUReadPort`/`S9xAPUWritePort` run
`S9xAPUExecute` first and access the port only at that synchronized boundary,
and HMax runs `S9xAPUEndScanline`. The pinned NTSC ratio, its remainder and
the signed SMP credit/debt are the only clock model: nothing resets the APU,
seeds a lookahead, opens a host output window or manufactures a cold
checkpoint. A negative `smp_clock` seed is refused, because a completed
instruction boundary cannot owe execution `S9xAPUExecute` already performed.
`ApuHostPortProbe::from_snes9x_coroutine` adds the second provenance — a
machine already in coroutine form, whose exact DSP owner `end_scanline_at`
drains where `SNES::dsp.synchronize()` does. `RomCpuTimingProbe` adopts an
owner only through `attach_apu_port_owner`, which refuses a clock reference
ahead of its own CPU master clock; without an owner `$2140..$217f` still fails
closed. APUI accesses synchronize to the access's *start* timestamp, because
pinned `getset.h:S9xGetByte` runs the register semantic before
`addCyclesInMemoryAccess`.

The beam-counter owner is now shared (`fdcd2f0d`). `CpuSynchronousMachine`'s
lone `source_ppu_open_bus1` became the whole `SourcePpuReadState`, so the exact
cold executor no longer rejects `$2137`, `$213c`, `$213d`, `$213f`, `$4213` or
`$4201`, and both executors use one counter/open-bus implementation. Its cold
seed is `ppu.cpp:S9xSoftResetPPU` (WRIO/RDIO `$ff`), so the SLHV gate comes
from the documented reset state rather than an assumption. The quiescent
checkpoint is version 6 and rejects 5. STAT77's `PPU.RangeTimeOver` still has
no owner and keeps failing closed.

**Recovered fixture — the three "lost SRAM" proofs run again.** They need a
save-present cartridge image: the boot's `$00:87EF LDA $7003E5 : CMP #$55AA :
BEQ` reads `$55AA` and takes the branch, which costs the `cpumacro.h:bOP`
ONE_CYCLE the recorded transaction stream charges. Every `initial.srm` under
`routes/` is instead a fresh `$60` fill, so seeding one makes that branch fall
through and every later transaction disagree — that, not a timing bug, is what
substituting the route seed produces.

The image the fixture was actually captured with (SHA-256 `d8a02e6e...`) is
still gone. `saves/sram.dat` is a *different* save-present image (SHA-256
`71b9a4021b8329ac8ca9567febfa018fc5020059f62bafbf70d42bad31bf5f28`) that
reproduces every compared transaction through 1,000 host calls, so it is
equivalent for these proofs — it is not the recorded capture seed, and a proof
that reads further into the save block could legitimately need the original.
It has been copied to
`routes/full_run/comparisons/continuous-audio/initial.srm` so the proofs bind
to a stable fixture rather than to a live save file that the game rewrites.
`route_initial_sram()` tries `ZELDA3_ROUTE_SRAM`, then that path, then
`saves/sram.dat`, and asserts the `$55AA` marker so a wrong seed reports
itself. With it, all seven external-ROM tests pass — including 1,000
continuous host calls of the real ROM through the exact cold executor, which
is what covers the new counter owner.

The exact cold executor now shares a source-ordered active-HDMA stall with the
timing probe, retains enabled HDMA across quiescent checkpoints, reads HVBJOY
from the processed beam cursor, and implements the 65816 instruction families
encountered on the recorded take. The eight `(direct page,X)` ALU/store forms
share one pinned indexed-pointer sequence; later direct-indexed, indirect-Y,
shift/rotate, test/set/reset, and absolute-indexed forms retain their distinct
bus and width behavior. The opt-in source OAM port owns `$2138` storage and
buffering. A source OBJ evaluator and H=512 render event now supply STAT77
`$213E`'s accumulated range/time-over bits in source event order. The cold
owner also implements the multiplication register transaction used by the
take. Its quiescent checkpoint is version 9, including the presented OAM
snapshot. Normal native `Snes` instances
do not enable the source OAM port.

The capability probe completes all 6,277 host calls of take 0000 without an
unsupported CPU/MMIO operation:
`cargo run -p snes --example source_route_probe -- zelda3.sfc routes/full_run/comparisons/continuous-audio/initial.srm routes/full_run/takes/0000/input.txt 6277`.
That is **execution coverage, not source parity**. The existing cold-owner
1,000-call test runs neutral input; its fixed timing/return witness cannot
validate the recorded take after its first nonzero input at host call 68. A
same-event source receipt is needed to validate the take's timing, register
reads, and game state. The cached oracle's `presented_oam` is captured at
presentation, so comparing it with the cold owner's live OAM at a main-loop
return cannot establish an OAM divergence. The available SRAM is proven equivalent to the deleted
capture seed only through host call 1,000. Production native still uses
`RomCpuTimingRun`; promoting the exact owner requires retained handoff into
native execution and source receipt comparison across that transition. The
1,581,079-frame A/V gate is a separate production-native acceptance check.

The cached whole-route oracle has a different initial SRAM image (SHA-256
`a6af0ddf25ae5feafba301100938f5ef81137ee76dcdc483a04fb6e961347828`)
from the source-route fixture above (SHA-256 `71b9a402...`). Comparing their
OAM after host call 1,000 misattributes save-dependent game state to CPU/PPU
logic. The probe now optionally reads the cached oracle's compressed host
receipts and compares **presented OAM at the same VBlank capture event**.
With that oracle's exact SRAM and recorded input, all 544 presented OAM bytes
originally matched for host calls 0..=4,399. The next byte came from a Link Y
write after input `$0030`: pinned libretro reports Up before Down and, with
opposing directions disabled, Down wins. The source owner had sent both bits
directly to the JOYSER latch. `set_libretro_joypad_words` now applies the pinned
libretro control rule for both opposing-direction pairs and both ports at the
input boundary. The raw serial-state setter keeps its original semantics. The
source owner then matches every presented OAM byte on **all 6,277 calls of
take 0000**. Before the CLI IRQ-selection fix, the longer cached whole-route
run matched all 544 presented OAM bytes through host call **413,641**, using
that oracle's exact SRAM and recorded input. Its first difference was host
413,642: OAM offset `$35` was
`$4D` from the source owner and `$52` from Snes9x; the other 543 bytes
match. The source CPU stores `$4D` into WRAM `$0835` at `$08:F6F9` on calls
413,641 and 413,642, so the first observed error is upstream of the OAM
port. The ROM instruction at `$08:F6F9` is an indirect store of a previously
computed value. At the same indexed polyhedral table load in call 413,641,
the source reads `$5D` from `$08:0C02` while Snes9x reads `$62`. Both execute
the table store at `$09:AD08` during call 413,640, with different computed
accumulators. The first **1,405** ordered writes to that mirrored WRAM byte
match; this is the first unequal write. The immediate differing operand is
the preceding `GetRandomNumber` read of `$213C` at `$0D:BA74`: source gets
`$23`, Snes9x gets `$28`. The call-413,640 instruction traces show an earlier
register difference at `$00:82CB`: the interrupt return path reads `$1F2F`
from WRAM `$1F0A` in the source owner and `$1F2E` in Snes9x. The source wrote
`$1F2F` at `$00:832A` on call 413,639; the oracle wrote `$1F2E` at the same
ROM store (the trace records its post-store PC `$00:832D`). This resumed capture matched
the cold oracle's video and audio hashes on both calls 413,639 and 413,640.
That saved value changes the restored stack pointer and return path. Its
source-owner divergence predates the random read and still needs tracing to
the first differing instruction boundary. The source enters the V48 interrupt
after the idle-loop load at `$09:F81D`; Snes9x enters after the following
branch at `$09:F81F`. Their RTI targets are `$00:8034` and `$00:8036`, respectively,
leaving source 22 master cycles earlier at the `$213C` read. A 42-cycle HDMA
stall then lands during source's `$2137` read but before the oracle's
callee entry. These are downstream effects of a source IRQ-acceptance error,
not reasons to adjust the RNG value or force an extra idle instruction.
This is a source-owner presentation witness, not a production native A/V
parity result.

A pre-fix cold source/oracle CPU-return comparison through host 413,639 checked
PC, V/H, A/X/Y, stack pointer, and status flags at each host boundary. Following
the 15 previously known transient differences on calls 0..667, all eight
fields match on every call 668..159,582, then differ on 33 calls in
159,583..159,849. They match again on every call 159,850..413,625. The next
11 differing returns occur in 413,626..413,639, beginning with a different
idle-loop PC and beam cycle at 413,626. Full instruction traces place the
first difference at `$09:FB6B` (`CLI`) in call 413,626. Both machines enter
that instruction at V48/C4 with identical registers and timing. The source
executor used CLI's newly cleared I flag to select the pending IRQ at the
same boundary; pinned Snes9x checks IRQ against the previous I flag, then
publishes the CLI change and executes `$09:FB6C` (`RTS`) before entering IRQ.
The central executor now selects IRQ with the pre-CLI/SEI I flag while
retaining the new flag for pushed status and following instructions. From the
matched call-413,625 checkpoint, this rule makes all 11,937 instruction PCs,
post-instruction A/X/Y/stack/status values, and instruction-start master
times exact on call 413,626; its return PC/V/H and registers also match.
The new cold OAM replay from reset matches **all 544 presented bytes through
500,000 host calls** using the cached oracle's identical ROM, SRAM, and input.
This advances the source presentation witness beyond the former call-413,642
failure. Neither the focused checkpoint nor the source OAM witness promotes
the production native A/V frontier.

A SHA-bound source checkpoint at host 413,625 was resumed through host
500,000, and the resulting host-499,999 checkpoint was resumed through host
600,000. Both resumed segments matched all 544 presented OAM bytes on every
host call. The source probe now accepts `ZELDA3_SOURCE_PROGRESS_EVERY` to
report its current host during long comparisons. The checkpoints and logs are
under `target/source-cli-fixed-*`; this extends the source OAM witness, not
the production native A/V result.

A new cold replay from reset matches the oracle's PC, V/H, A/X/Y, stack
pointer, and status flags at **every one of the first 160,000 host returns**.
This includes the opening and call-159,583 return differences from the
pre-fix run.

At the route endpoint, save a source checkpoint after host 1,581,078 and run
`python3 scripts/compare_source_cpu_final_state.py SOURCE_CHECKPOINT ORACLE_CACHE`.
The comparator verifies ROM/SRAM/input hashes against the cache manifest and
the oracle snapshot hash before comparing CPU registers, all 128 KiB of WRAM,
and the 8 KiB cartridge SRAM. It does not compare PPU or APU internals; a
matching OAM route alone must not be described as full CPU-state parity.

The separate live Snes9x frame trace gives a stricter CPU-return witness.
Through take 0000, source and oracle return PC/V/H now differ on only 15
transient calls in 0..667; every call 668..6,276 matches the return PC and
beam position. The first sustained difference had been an eight-master-cycle
shift at host call 5,722. Full PC tracing located it at `JSL $05:B5C3`
(`$06:BFEA`, V15/H1066): the source charged 58 clocks to terminate an
indirect HDMA channel where pinned Snes9x charged 50. The final active
channel's zero descriptor uses one indirect fetch cycle and reads from the
descriptor address itself; a higher active channel retains two cycles and
reads from the following address. Both HDMA initialization and per-line
reload now follow those rules. At host call 5,722 all 11,490 traced
instruction PCs and all 11,219 active-frame PC beam positions match the
oracle. The source additionally needed pinned LoROM open-bus reads, ignored
writes to read-only `$4210..$421f`, mirrored DMA register reads, LoROM
open-bus holes, the live `$2180..$2183` WRAM port, and the encountered LSR,
ROR, ADC `[dp]`, LDY, and STY instruction forms to continue the recorded
route. OAM agreement does
**not** establish exact CPU timing beyond the measured return/trace interval
or native production parity. Next compare source register/WRAM and return
timing against same-event oracle receipts beyond take 0000, then integrate
the retained source CPU into production native execution without substituting
probe-only values.

A rebuilt production `zelda3` binary (SHA-256
`a790c35bbd13294ea370ab3d7d4bd37dc37a65f76d2af26ee04c62aba055c4fe`)
matched the cached recorded-receipt route for **1,581,079 contiguous exact
video and audio frames**, from frame zero, in
`target/source-cpu-central-full-preservation` (1,619.59s). The independent
cold native-timing gate on the same binary again reached the established
first-video-mismatch frame **56,458** (1,684 mismatched pixels; audio had not
failed). The existing counter-read continuation analysis above remains the
next native integration target; the source-owner OAM witness and the
receipt-driven full-route proof cannot substitute for native-timing parity.

Evidence: `retained_continuation_port_timing_matches_the_pinned_cold_ipl_handshake`
reproduces every recorded CPU/APU handshake access through the first CC from a
real IPL execution; `local_rom_probe_apu_ports_match_the_pinned_cold_boot_writes`
seeds the probe only at the pinned `S9xSoftResetCPU` boundary, runs the ROM's
own boot, and lands its four `$2140..$2143` writes at the recorded
`(v_counter, cpu_cycle)` with the recorded `apu_cycle_after` before failing
closed at `$00:8018`'s unowned `$2100` write — synchronizing six clocks later
instead fails it at write 1 (APU 18 vs the recorded 16), so the ordering is
load-bearing. 423 snes tests pass (7 ignored, all seven run and pass with the
ROM and the recovered SRAM); 1,799 zelda3 tests pass, 3 ignored.

Native A/V re-measured on the rebuilt binary (SHA
`4b72d7d3909b7e174b6dc315fc99ad8ab1f1930486acecc5d7979a15fc33d6a6`,
`target/native-counter-apui-smoke`, 70.44s): exact through 56,389, first video
mismatch 56,390, audio exact — identical to the previous binary, as a
runtime-unreachable change must be. No full 1,581,079-frame run was made.

What is still unowned, in the order the native route needs it:

1. **The route's APUI hand-over.** `RomCpuTimingRun::new` copies four
   `apu_output_ports` bytes into a fresh shadow. `ApuHostPortTiming` needs the
   engine's live APU and its clock provenance handed over and returned, not a
   copied latch.
2. **PPU register writes in `RomCpuTimingProbe`.** It fails closed at the very
   first `$2100` write, which is why its cold-boot witness stops at `$00:8018`.
   The exact cold executor already owns `$2100..$21ff` through `snes.write`.
3. **Automatic NMI dispatch and auto-joypad in the probe.** Its HMax handler
   only moves `in_vblank`/`in_nmi`; the cold executor's already publishes RDNMI,
   schedules the H=12 acceptance deadline and runs the V228 auto-read. It also
   needs a real NMITIMEN owner — the probe currently accepts only the no-op
   `$4200` write `S9xSetCPU` takes via its `Byte == FillRAM[0x4200]` early-out.
4. **DMA/HDMA execution in the probe.** The cold executor already models
   general DMA; the probe rejects every nonzero enable.

The duplication in 2-4 is worth weighing against finishing the probe: the exact
cold executor is far more complete and now reads counters too, so the remaining
gap between it and the native route is its *seed* — it constructs only from a
cold LoROM reset or a quiescent checkpoint, never from an arbitrary route
frame. Closing that seed may be cheaper than re-deriving PPU writes, NMI
dispatch and DMA in the probe.

### Next: fix bus access timing and remaining CPU-phase ownership

The confirmed beam-counter defect below still samples `$2137` at instruction
entry rather than its source bus-access time. Fix ordered bus timing, with
source-backed access timestamps and stall handling; do not inject cached RNG
or add an RNG-specific24-clock adjustment. Dialogue initialization and
interrupted dialogue returns still do not retain every CPU phase; fresh
completed iterations are the scope of this batch. Continue tracing those
ownership boundaries before interpreting a later OAM interruption as a new
missing hold. Earlier failures are acceptable evidence when source fidelity
improves; exact coverage alone must never justify a timing shortcut.

### Source-ordered timing probe: original RNG/Cucco path now proven in isolation

`RomCpuTimingProbe` is a separate development bus owner using the shared
instruction layer. It accepts an explicit SNES machine, physical timeline,
and `SourcePpuReadState`; it is not an alternate constructor for the exact
cold CPU/APU executor. It currently supports the audited LoROM/WRAM/SRAM
map, Mode7 product reads, WRIO/RDIO, and the counter/status read subset.
Unsupported I/O fails closed and poisons the probe. Pending interrupts,
DMA/HDMA work, ambiguous refresh seeds, and invalid cartridge/counter seeds
are rejected. APUI is now supplied, but only through an explicitly adopted
`ApuHostPortTiming` (see the batch section above); automatic NMI dispatch and
DMA/HDMA execution are still not supplied by this owner, so it has not replaced
the native route's aggregate probe.

The next batch shares the source native interrupt-entry bus sequence with
this probe. `accept_native_nmi` requires an already accepted interrupt at
an instruction boundary; it does not invent VBlank or schedule NMI deadlines.
A separate interrupt receipt records the stack/vector transactions without
a fake opcode. SlowROM/FastROM entry takes62/60 clocks; RTI restores the
original bank, PC, status and stack. RDNMI acknowledgement belongs only to
reading `$4210`. Zero writes to inactive DMA/HDMA enables are accepted;
nonzero enables still fail before activating DMA.

Validation: `/tmp/native-probe-nmi-tests.log` has415 passing SNES tests,
6 ignored, plus the integration test. The explicit original-ROM IPL test
passes in `/tmp/native-probe-nmi-ipl-proof.log`. The development witness
`target/native-source-nmi-prefix/{probe.rs,probe.log,source-prefix.json}`
loads the original ROM and recorded source WRAM, then matches the source
NMI prefix's PC, raster, A/X/Y, SP and status at every traced instruction
from `$80c9` V225/C76 through `$80e1` V225/C442. It fails closed at the
actual APUI read V225/C466. This is a seeded prefix witness, not proof of
interrupt acceptance, whole-handler execution or native route coverage.

That APUI owner now exists as `ApuHostPortTiming` (see the batch section
above); the constraints it was written under still hold for anyone extending
it. `AbsoluteDspEventClock::advance` starts a host audio window and increments
the host index; repeatedly calling it for CPU bus accesses would be wrong.
Its legacy APU state is also not an exact Snes9x SMP coroutine checkpoint.
Do not seed frozen acknowledgement ports, reset a late APU, or manufacture
an exact checkpoint to bypass an ownership boundary. What remains unowned is
the *route's* APUI: `RomCpuTimingRun::new` still copies four `apu_output_ports`
bytes into a fresh shadow instead of handing over the engine's live APU, so
integrating `ApuHostPortTiming` there needs an explicit hand-over-and-return
boundary for that machine and its clock provenance.

A further SPC queue audit reproduced lost future CPU writes: `advance`
took the complete scheduled-write queue but discarded the unconsumed suffix
when the output window ended. It now retains that suffix at its original
absolute timestamps. Consumed writes remain exclusively owned by the APU
scheduler or published latches. The regression executes SPC `MOV A,$f4;
MOV $f5,A; BRA` and verifies delayed publication, an actual port echo,
identical PC/cycles after split versus uninterrupted windows, and no replay.
It fails on the previous implementation with input0 instead of$ff:
`/tmp/native-future-apui-baseline.log`. The source contract is
`apu/apu.cpp:S9xAPUWritePort`: synchronize the APU, then publish the input
latch; an audio output boundary cannot cancel a pending CPU write.
The queue-fix binary SHA is
`4f1b67a1880bff62b410501e1ed09b4b6e6acc85ab836f6fd52fbc6bfbd419db`.
`target/native-future-apui` remains native exact through56,389, first video
failure56,390 with audio exact (458.97s). The same binary matches all56,397
receipt-driven frames in `target/native-future-apui-receipt` (367.97s).
No full1,581,079-frame comparison was repeated.
All1,799 library tests pass,3 ignored (23.69s):
`/tmp/native-future-apui-tests.log`. This queue repair does not yet supply
fine-grained APUI sampling or remove the host-window semantics of `advance`.
The legacy clock also retains nominal NMI entryC84 and SPC lookahead19.
Neither is a valid replacement for the source CPU phase and retained SMP
clock/remainder at an arbitrary bus access. Do not tune these constants;
prove and preserve the actual synchronization state when integrating APUI.

`apu::ApuHostPortProbe` now supplies retained native SPC instruction ownership
for that future adapter. Its constructor consumes an explicit completed
legacy APU instruction boundary, rejecting pending legacy cycles, another
coroutine, and an exact DSP owner. It preserves the actual RAM, timers,
ports and scheduled events, with no reset or CPU/SMP clock seed. Each step
uses the existing source pseudo-op executor. A suspended or poisoned probe
cannot return its machine to legacy instruction execution; refusal retains
the entire probe. CPU port publication is explicit and does not advance time.
This type is deliberately not serializable and is not a cold checkpoint.

`/tmp/native-apu-port-probe-tests.log`:417 SNES tests pass,6 ignored, plus
integration. The regression uses the pinned IPL fixture: suspend at2398
before AA, refuse a partial machine handoff, then resume AA/BB at the source
store cycles. Scheduled input events survive, and the final RAM, ports,
PC, cycle count and instruction duration agree with complete-instruction
execution. The existing runtime callers are unchanged; this new capability
still needs a proven CPU-to-SMP synchronization owner before native integration.

The queue-fix A/V binary above predates this isolated API addition. No A/V
run is claimed for a binary rebuilt with this probe; runtime callers have
not been switched to it.

Counter state explicitly owns WRIO, PPU.OpenBus1/2, latchedH/V, read flips,
and the STAT78 latch flag. The reset factory follows `S9xSoftResetPPU`
(WRIO=$ff); it must not be used as a guessed later-frame seed. Source read
semantics now cover gatedSLHV, forced WRIO falling-edge latching, the two
long dots, the odd-field short line, high-byte OpenBus2 retention, and
STAT78's model3/field/latch bits and flip reset. CPU OpenBus remains separate.
This subset assumes NTSC with no pending light-gun latch.

`CpuMasterTimeline::synchronous_beam_position` uses the unconsumed source
event cursor. An opcode fetch can physically cross HMax before `CPU.V_Counter`
advances; normalizing the absolute timestamp alone loses that distinction.
Regressions cover ordinary HMax, the short scanline240, and field wrap.

The ignored local-ROM test
`local_rom_counter_probe_matches_source_cucco_branch` executes the actual
`$0d:ba71` RNG routine, RTL, `$06:a7f9 STA $0f`, AND, and BEQ with the
recorded source stack/register/RAM inputs. It asserts every owned write,
register preservation, returned PC and exact raster. No RNG result is fed
into execution. `/tmp/native-timing-probe-rom-witness2.log` records:

| Input phase | Computed RNG | Next PC | End raster |
| --- | --- | --- | --- |
| SourceV103/C1168 | $32 | $06:a7ff | V104/C52 |
| Earlier nativeV103/C1156 | $2f | $06:a7ff | V104/C40 |

Both cases pass with initial PPU.OpenBus2=$00 and$ff, proving the prior value
is irrelevant to this low-byte read. Source comparison56,389 in
`/tmp/native-56390-cucco-source.jsonl` independently records `$a7f9` at
V103/C1360, `$a7fb` atV104/C20, `$a7fd` atC36 and `$a7ff` atC52.
The old aggregate-interpreter reproduction at the exact source entry gave
RNG$2c and therefore the other branch. The new probe fixes that isolated
bus-access cause while preserving the remaining12-clock entry difference;
it does not establish a new native A/V frontier.

The bus audit also fixed accepted FastROM operands in the exact executor and
new probe: immediate8/16/long now use the active opcode's MemSpeed instead of
hard-coded SlowROM8/16/24 clocks. Tests assert a FastROM immediate word's
6+12 transaction sequence and a counter read's6+12+6 sequence. Direct PCBase
operands at the bank-end switch to an unimplemented slow-path variant now
fail closed rather than silently wrapping through a different memory map.

Validation:413 SNES library tests plus the integration test pass,6 optional
local-ROM tests ignored by the ordinary suite:
`/tmp/native-timing-probe-all-tests2.log`. The external-ROM RNG/Cucco test was
run explicitly and passes; the exact cold executor's recorded3.2M-transaction
IPL proof also passes: `/tmp/native-timing-probe-ipl-proof.log`.
`cargo check --profile parity -p zelda3` passes (17.37s):
`/tmp/native-timing-probe-integration-check.log`. No frozen
fixtures, ROM files, or runtime timing offsets were introduced. No native or
receipt A/V run was repeated because this probe is not connected to that path.

Next establish the native caller's authoritative PPU-read/timeline seed and
an explicit NMI/APUI/DMA ownership boundary before integration. Reuse real
source handler transactions or a source-proven handoff; do not bypass the
constructor checks, switch interpreters only around RNG, supply cached RNG, or seed
an arbitrary later caller with reset values. Then batch the source opcode
and hardware coverage exposed by a frame-zero native run toward100k.

### Shared source instruction execution and optimized completion ownership

`crates/snes/src/cpu_synchronous_executor/source_cpu/instruction_set.rs` now
owns the37 source opcode/address/stack/ALU methods behind a private
`SourceCpuInstructionBus` interface. The existing cold executor implements
that interface through its original immediate/getset/AddCycles methods.
Its complete CPU/APU/timeline seed, event draining, poison state and
checkpoint validation remain with that owner. No raw shadow seed API was
added. `target/native-source-instruction-extraction/mechanical-comparison.json`
compares all37 methods to the prior code after only accessor/format
normalization, with no remaining differences.

A separate test-only bus executes original `$0d:ba71..ba7e` with the recorded
comparison56,389 register/RAM inputs. The shared instructions sample
`$2137` atC1192, `$213c` atC1222, and write RNG$32 atC1308, endingC1316.
The expected RNG is asserted, never supplied to the computation. This is
a scoped low-counter source witness, not a runtime PPU seed or a complete
counter model. A second test starts an absolute read atC522: the non-draining
opcode fetch adds8, the word operand transaction adds16 and observes refresh
atC546; the register is sampled atC586 after the40-clock refresh. Together
these exercise a bus owner independent of the exact CPU/APU constructor.

This validation exposed a real optimized-build ownership defect, also
reproduced on unchanged baseline38143608. Seven successful completion
retirements were hidden inside `debug_assert_eq!`; release/parity builds
therefore left `pending_completion` populated and rejected the following
instruction with `PendingCompletionMustResume { completion: Write }`.
Those retirements now use unconditional assertions, so byte/word reads and
writes, APUI writes and completed general-DMA resumptions consume their
completion exactly once in every build profile. Failed event drains still
retain their existing completion for explicit resumption. A consecutive
LDA/STA/REP/SEP regression checks byte and word ownership, all writes, final
registers, and the exact198..408 CPU-clock interval.

Validation: all404 SNES library tests and the integration test pass,5 local
ROM tests ignored by the ordinary suite:
`/tmp/native-source-completion-all-tests.log`. The local-ROM
`local_zelda_rom_matches_every_timing_transaction_through_final_ipl_handoff`
was then run explicitly and passes its recorded3.2M transaction comparison:
`/tmp/native-source-completion-ipl-proof.log`. Its pre-fix baseline failure
is `/tmp/native-source-instructions-ipl-baseline.log`. No fixtures or ROM
bytes were regenerated, and the temporary ROM symlinks were removed.

The native timing probe still uses the legacy aggregate interpreter. Its
56,390 video frontier has not been remeasured or improved by this extraction;
no native or receipt A/V run was spent on this isolated source-executor work.
Next implement the timing-probe bus owner against this shared instruction
layer, with explicit physical event/counter state, retaining all seed and
unsupported-hardware boundaries described below. Do not enable the incomplete
counter model merely because the isolated RNG witness now matches.

### Counter audit: a bus-clock fix also needs a valid hardware seed

`target/native-counter-contract/{probe.rs,probe.log,manifest.json}` records
four CPU-only reproductions against the unchanged current SNES library.
Expected values below come from pinned `source/ppu.cpp:S9xLatchCounters` and
`S9xGetPPU`, not an additional live-core run:

| Contract | Source-derived expectation | Current Rust observation |
| --- | --- | --- |
| `$2137` with WRIO bit7 low, previousH=$123, currentH=100 | retain291 | overwrites with100 |
| `$4201` bit7 falling, same old/current counter values | latch100 | retains291 |
| Normal scanline103 atC1292 | horizontal counter322 | counter323 |
| `$213c` low then high fromH=$123 | low$23, high$23 (PPU.OpenBus2 bits7..1 retained) | low$23, high$01 |

These are independent counter-contract defects; they are not newly proven
causes of frame56,390. That frame's first low-byte read atC1192 is before the
long dots, and its source instruction-start sampling defect remains the
separate, executable RNG reproduction below.

These four rows describe the **live `Snes`/`PpuState` bus**, which the native
route's aggregate `RomCpuTimingRun` uses. They are still open there. The exact
cold executor and the source-ordered probe no longer share that bus: both now
read counters through `SourcePpuReadState` (`fdcd2f0d`), which implements every
row. Fixing the live bus is therefore a separate, runtime-reachable change;
do not assume the shared owner already covers it.

The relevant ownership gaps are concrete. `Snes::read_b_bus($37)` currently
latches unconditionally with `h_pos/4` and returns CPU OpenBus. The source
checks WRIO bit7, uses the physical scanline's long-dot conversion, and
returns PPU.OpenBus1. `Snes::write_reg($4201)` calls `ppu.read($37)`, whose
match does not latch anything. `PpuState::read_latched_counter` drops the
retained PPU.OpenBus2 high bits. Its `$213f` path resets the flip-flops but
returns a placeholder$ff instead of owning the source status/open-bus state.

Do not simply turn on the missing WRIO gate: `RomCpuTimingRun::new` creates
`Snes::new`, clones RAM/PPU/DMA and the four APU output-port bytes, but does
not seed WRIO; `Snes::ppu_latch` therefore starts false while `S9xResetPPU`
sets `FillRAM[$4201] = FillRAM[$4213] = $ff`. Gating SLHV on a field that
starts wrong would stop the route latching at all. Correct gating requires
authoritative hardware state carried into the shadow, rather than assuming it
high to keep RNG working. Physical field, PPU open buses, and latch/read-flip
state must also have explicit owners before constructing an exact counter seed.
`Snes::ppu_latch` is a one-bit model of WRIO bit 7 and `$4213` returns only
that bit, so a full RDIO byte needs an owner too. The cold executor shows the
shape this should take: one `SourcePpuReadState` seeded from the documented
reset, not a scatter of fields on `Snes`.

The timing architecture has a separate constraint. `RomCpuTimingRun::step`
executes all instruction semantics first;
`advance_rom_cpu_step_measured` subsequently drains the aggregate CPU work,
HDMA and general DMA. Adding a bus-access prefix to the raster cannot fully
reproduce pinned `PCBase` fetches, word operand transactions, internal cycles,
and event draining. The existing `source_cpu.rs` executor already models
those transaction boundaries, but its audited read map intentionally rejects
`$2137/$213c` and its quiescent seed is a complete CPU/APU/timeline owner.
Do not bypass that seed contract to transplant a partial shadow. Reuse the
source transaction semantics through an explicit timing-probe backend, or
extend the seed only with evidence for every hardware field it requires.
The counter semantics and source-ordered hardware access path need to be
validated together before claiming the RNG probe exact.

## Previous phase-retention regression — 54,043 (audio)

Interrupted ordinary-overworld HUD, Link-body, and sprite-preparation
suffixes now preserve the CPU's real next main-wait phase. The existing
probe executes the accepted NMI and remaining source instructions through
the common suffix and busy loop, then retains only the register checkpoint
and cycle budget for the next eligible host. Shadow RAM is never copied
into gameplay. Uninterrupted callers use the same suffix-to-wait helper.
The subsequent caller no longer silently falls back to `$8036`/H12 after
these typed interruptions. `ZELDA3_DEBUG_SONG_UPLOAD=1` now logs these as
`song_upload main_wait`, including the retained host, PC, raster and Z flag.

This removes an accidental timing cancellation: native first audio mismatch
returns to54,043, with video still exact there. The command at native host
54,020 now reachesV118/C246 versus source comparison54,019 C234 (12 clocks
late); the old fallback happened to reachC236. Do not compensate this with
a command offset. The unit regression uses the original `$8034 LDA $12` /
`$8036 BEQ $8034` bytes and source costs24/22: two paths reach the same
V225/H12 boundary with different next PCs, and both must survive unchanged
with their CPU flags/registers. Generic raster reseeding cannot represent
that distinction. Absolute caller phase still requires the upstream fixes
below; this is not a claim of improved native route coverage.

`target/native-overworld-suffix-phase` and
`target/native-overworld-suffix-phase-upload` reproduce this native frontier
from frame zero (68.74s/67.20s). Binary SHA:
`1a5a866384f39ccf97900f25654cb0770cb46e394b6bb79674b78ba821aaa1d5`.
All1,796 library tests pass,3 ignored (23.70s), including the new busy-loop
regression: `/tmp/native-overworld-suffix-phase-lib-tests.log`.
`target/native-overworld-suffix-phase-receipt` matches all56,397 receipt-driven
frames (63.81s). No full1,581,079-frame run was repeated for this binary.

### Evidence that motivated dialogue return-phase retention

The last non-suffix gap before this upload is dialogue14/2 returning to9/0.
`target/native-52752-source`, from the valid paired52000 checkpoint, is
video exact through53,926. `/tmp/native-52752-source.jsonl` shows source
comparison52,749 accepting a trailing NMI at `$8036`, V225/C22, counter165,
after returning to9/0; the callback returns at `$80c9`, C84.52,750 starts
inside that handler, advances counter166, and accepts the next NMI at
`$8034`, C14. The native ordinary probe's first retained phase after this
dialogue is host52,752 `$8036`/C28; the entry before it was a fallback.
Trace the dialogue caller's actual return and carry its phase into the
ordinary loop. Do not seed a frame-specific PC/raster from this observation.
Source WRAM52,748..52,756 is in that session; its decoded trace has both
frame entry/return and NMI records. Align counter and leading/trailing
ownership before equating native `host` with source comparison frame.

The independently confirmed beam-counter bus-sampling defect described below
also remains open. It needs actual access-time sampling, not RNG substitution.

## Previous native frontier — 56,390 (video): LinkOam body drawing

LinkOam's body drawing now has a native continuation between upper-entry
stores and lower-entry selection. The CPU probe recognizes the four ASL
instruction boundaries at `$0d:a9ed..$0d:a9f1`; it retains their executed
CPU work, not a frame/scanline exception. The translated prefix publishes
the upper entry and retains the lower entry's data and caller locals.
Resuming finishes the lower entry, visibility/stair return, HUD, rain, and
common suffix without replaying equipment or upper stores. The existing
HUD continuation is now named `FinishOverworldSuffixCallerReturn`, with
typed HUD and LinkBody variants sharing the same NMI/caller lifecycle.

`target/native-link-lower-body` proves native exact A/V through56,389,
first video mismatch56,390 with audio exact (80.92s).
Binary SHA: `d53f5df0048af99382e4cd4912678664c99a520e68493e756e71ab63aa7c8e0f`.
All1,795 library tests pass,3 ignored (23.94s):
`/tmp/native-link-lower-body-lib-tests.log`. The new regression covers
all five lower-selection edges, deferred lower stores, preservation of a
changed committed upper entry, and equal OAM/CPU totals after resumption.
The native run's `state-comparison.json` proves counter/latch/OAM agree
with source across54,758..54,766. Its WRAM dump extends through55,262;
the new frontier is beyond that diagnostic window.
`target/native-link-lower-body-receipt` matches all56,397 receipt-driven
frames (63.42s). No full1,581,079-frame check was repeated for this binary.

### Next: reconcile the next body interruption with the probe's CPU phase

`target/native-56390-source`, paired from53500, is video exact through
56,396. `/tmp/native-56390-source.jsonl` shows source56,388 reaching
the main wait with counter172 before its trailing NMI. Source56,389 ends at `$0d:aa05`, V225/C6,
counter173/latch1;56,390 accepts NMI at `$0d:aa08`, C52 and clears the
latch without advancing the counter. Native's probe instead reports
host56,389 `$0d:aab9`/C14 and host56,390 `$00:8605`/C24. These are not
the same interruption: compare native WRAM/entry phase before extending
the body continuation or changing packing. Diagnostic running/recorded
as `target/native-56390-diagnostic`, with presented captures in
`target/native-56390-diagnostic-presented`; source presented captures in
`target/native-56390-source-presented`. Do not add a late-return hold from
the native `$aab9` report without reconciling that earlier source phase.
`target/native-56390-diagnostic/state-comparison.json` confirms counters
and OAM agree through56,388 (native/source latch values differ during
the preceding ordinary cadence). At56,389 both counters173/latches1
agree, but native `$09a5=105` versus source240 and `$0a08=234` versus254.
Source PC `$aa05` is the lower XY STA, and `$aa08` is its following TXA:
the host return precedes that store, while NMI accepts after it. At56,390
native counter174 versus source173 precedes45 OAM byte differences.
The probe log's host is `frame_ctr_dbg`; account for its leading/trailing
NMI ownership before aligning it with comparison frames.

### Probe diagnosis: Cucco avenger RNG changes the measured path

The next diagnostic must reconcile the probe's CPU/beam/RNG input, not
extend Link's late-return continuation. `target/native-56390-sprite-phase`
and `target/native-56390-cucco-phase` both retain first video mismatch56,390
(70.05s and70.04s). No runtime fix landed from these probes.

For source comparison56,389 / native effective CPU host56,390, both enter
the main routine with counter173. Native is12 master clocks early at
`$02:a475`, Sprite_Main `$06:8328`, and sprite-slot13 entry `$06:84e2`.
The earlier slots preserve this difference. Slot13 is Cucco type$0b and
enters `Cucco_SummonAvenger` on this iteration. The source RNG call at
`$06:a7f5` returns$32 at `$06:a7f9`; its `AND #$02` branch takes `$a7ff`.
The native timing probe instead takes `$a823`. After the spawned avenger's
speed calculation returns at `$a84b`, native is708 clocks early; after
Sprite_Main returns it is682 early. These are different instruction paths,
not evidence for adding a fixed670/682-cycle charge or holding OAM.
The earlier12-clock phase difference may affect the RNG's beam-counter
reads, but the exact cause of the RNG disagreement is still unresolved.

Source `target/native-56390-cucco-source` resumes the valid paired53500
checkpoint and is video exact through56,391. Its decoded critical run is
`/tmp/native-56390-cucco-source.jsonl`; the earlier main/slot phase trace is
`/tmp/native-56390-phase-source.jsonl`. Joined source/native instructions
are saved in `target/native-56390-cucco-phase/instruction-comparison.json`.
Native pre-NMI WRAM is in
`target/native-56390-sprite-phase-input/56390-input-wram.bin`; sprite arrays
match source56,388 before this iteration. The temporary instrumentation
was removed from source; its reproducible patch is saved beside the joined
instructions as `diagnostic.patch`. The candidate binary still includes
that debug-only instrumentation; it is not a new gameplay revision.

Preceding ordinary iterations also expose an inherited phase difference:
native effective hosts56,387..56,390 reach main entry36,28,20,12 clocks
early respectively, compared with source56,386..56,389. Trace that phase
and the actual RNG register/counter accesses before deciding whether the
fix belongs to CPU timing, probe hardware state, or gameplay ownership.
Do not substitute cached RNG results into the timing probe simply to make
this branch match without establishing the intended input contract.

### Confirmed probe defect: beam-counter sampling uses instruction start

`target/native-56390-rng-source` is source video exact through56,391.
Decoded `/tmp/native-56390-rng-source.jsonl` proves the RNG's `$0d:ba71`
`LDA $2137` starts atV103/C1168, reads SLHV atC1192 (after three slow-ROM
fetches), and latchesH298. `$ba74` reads OPHCT atC1222 and obtains$2a;
with counter$ad, seed$5a, and entry carry1 the routine returns$32/carry1.

The current probe calls `cpu_run_opcode_timed`, which only accumulates bus
costs; it does not advance `Snes.h_pos` between memory accesses. Therefore
`snes.rs::read_b_bus($37)` latches the instruction-start position. A tiny
CPU-only reproduction in
`target/native-56390-rng-source/reproduce-counter-read.rs` links the current
`libsnes-c0fd764a432c4362.rlib` from `target/song-upload-build/parity/deps`.
Its executable/log beside it confirm RNG$29 at native startC1156, and$2c
even at the source's exact startC1168. This isolates a hardware-access
timing defect independently of the earlier12-clock phase difference.
The reproduction takes0.15s; use it before another route run. The original
ROM remains an external input. No gameplay or probe-timing fix has landed.

The correct implementation needs source-ordered counter sampling at the
actual CPU bus access, including intervening stalls; setting an RNG-specific
24-clock offset or supplying cached RNG output would hide the missing bus
model. The legacy interpreter's timed API currently reports only aggregate
instruction cost. The separate opt-in `cpu_synchronous_executor/source_cpu.rs`
has source-ordered transactions, but has a deliberately constrained seed
and supported-hardware contract; do not bypass those constraints to graft
it onto the probe. Establish the bus-clock ownership before changing it.

`target/native-56390-nmi-phase/instruction-comparison.json` joins109 handler
PCs with `target/native-56390-nmi-source`. The instruction paths match and
native remains10 clocks late from NMI handler entry `$80c9` to its return;
one temporary40-clock difference at `$81b5` reconverges at `$81b8` because
refresh falls in a different instruction. There is no missing NMI/DMA
charge in this handler. Native resumes the main wait at `$8034`; source
resumes `$8036` and takes the22-clock branch back to `$8034`, explaining
the resulting12-clock lead at main entry. Investigate retained busy-loop
phase separately, especially the fallback after an interrupted suffix.
The NMI diagnostic again first fails video56,390 (70.79s). Its temporary
instrumentation was removed and saved as `diagnostic.patch` beside the
comparison; the candidate executable still includes that debug-only probe.

### Previous native frontier — 54,762 (video)

The native HUD probe now retains a typed `BeforeHearts` interruption when
NMI arrives at `$0d:fb94`, before the hearts block executes. The resumed
callee performs hearts, magic, inventory, and the HUD flag once, then
returns through the existing rain/common-suffix continuation. It does not
repeat resource refill or represent hearts as an inventory digit field.
A regression checks deferred HUD tiles, one rupee update, the return flag,
and equal aggregate CPU cost against uninterrupted execution.

`target/native-hud-hearts-entry` proves native exact A/V through54,761,
first video mismatch54,762 with audio exact (78.80s).
Binary SHA: `5ee9d2d12362e9ebf56ba3db58adde757dc8b61e1aac1bfb9cb24f4c726a73d2`.
All1,794 library tests pass,3 ignored (23.70s):
`/tmp/native-hud-hearts-entry-lib-tests.log`.
`target/native-hud-hearts-entry-receipt` matches all54,767 receipt-driven
frames (62.69s). No full1,581,079-frame run was repeated for this binary.
`state-comparison.json` in that native run proves counter, latch, and OAM
agree with source across54,739..54,745. The probe reaches `$0d:fb94` at
V225/C34 versus source C20; no14-cycle offset was applied.

### Next: Link body OAM interruption before the lower entry

Source `target/native-54762-source`, paired from53500, is video exact
through54,766. `/tmp/native-54762-source.jsonl` shows comparison54,761
ending at `$0d:a9ef`, V225/C8, counter130/latch1. The next callback
accepts NMI at `$0d:a9f0`, C22 and eventually clears the latch with the
same counter. Native's probe reports `$0d:a9ef`, C22 on host54,762,
but currently supplies no LinkOam continuation for this ordinary caller.
`player_oam.rs::link_oam_after_equipment` draws both body entries
atomically; the ROM is selecting the lower entry after upper stores.
Retain its actual locals and committed body stores, then resume the
remaining body/blink/water-grass/HUD suffix. Do not replay equipment,
freeze all OAM, or add a host delay. Capture native WRAM at this window
in the next candidate run; the current native WRAM dump ends54,745.

### Previous native frontier — 54,741 (video)

The native overworld upload return now probes its common main-loop suffix
from the measured $4200 restore position. The remaining STA bus cycle,
RTS, and router RTL take6+42+44 CPU clocks; the real JSR and sprite
preparation instructions then determine whether NMI interrupts the suffix.
The probe copies no RAM into gameplay. Its progress selects the existing
partial sprite-preparation implementation and a typed upload-return
continuation; that continuation clears the latch after resuming without
consuming a synthetic trailing NMI or starting another main iteration.

Source `target/native-upload-suffix-source/cpu-boundaries.json` proves
comparison54,250: STA atV220/C986, RTS1016, common JSR1102,
sprite-preparation entry1148, NMI at `$00:864f`, V225/C12, Y12/X48.
The native probe reaches exactly that PC and raster position, with
group12/three committed bytes/972 CPU clocks of that group. There is
no scanline threshold, extra hold count, or OAM-generation freeze.

`target/native-upload-suffix` proves exact native A/V through54,740,
first video mismatch54,741 with audio exact (79.48s).
Binary SHA: `4354b7dc3bb5a0e594d4b050f5036df0294eaf06b1525972720de4f5a329bf12`.
All1,793 library tests pass,3 ignored (23.76s), including the new
upload-suffix return/latch/main-wait regression:
`/tmp/native-upload-suffix-lib-tests.log`.
`target/native-upload-suffix-receipt` matches all54,746 receipt-driven
frames (62.05s). The older frozen binary still owns the last full-route
proof; no1,581,079-frame check was repeated for this batch.
Its `caller-state-comparison.json` confirms main module/submodule,
counter, and radius agree across54,249..54,278. The premature54,250
latch clear is gone. Later opening-iris latch differences remain at
54,275/77 while video stays exact; do not call this whole-WRAM parity.

### Next: the HUD hearts block's fresh-entry interruption

Source `target/native-54741-source`, paired from53500, is video exact
through54,745. `/tmp/native-54741-source.jsonl` shows comparison54,740
NMI at `$0d:fb94`, V225/C20: entry to Hud_Update_IgnoreItemBox's hearts
block, before its writes. Source counter110/latch1 persists that host;
the suffix returns in54,741 with counter110/latch0. The current native
HUD interruption probe covers inventory entries/conversions, but does
not retain this pre-hearts entry. `target/native-54741-diagnostic` confirms
native clears the latch early in54,740, then increments the counter to111
in54,741 while source stays110. HUD tile words still match; OAM differs
in2 bytes at54,740 and42 bytes at54,741. Preserve the CPU call boundary
with a typed pre-hearts continuation. Do not treat this as an inventory field or skip/replay
already executed refill/item-box work.

### Previous native frontier — 54,276 (video)

Positioned native dungeon uploads now suspend their caller until the
executing SPC receiver completes its handshake. They bypass the fixed
22-host scheduler estimate, use the shared upload-return forecast, then
run the dungeon ambient-sound and common main-loop suffix once. The
translated engine remains the state owner. No hold count was retuned.

`target/native-dungeon-upload-return` proves exact native A/V through
54,275, first video mismatch 54,276 with audio exact (78.20s).
Binary SHA: `57b4c3f31c905de42cc1d40f7de0a7dd514c48d6744c9205b819642a3b61e311`.
All 1,792 library tests pass, 3 ignored (23.99s):
`/tmp/native-dungeon-upload-return-lib-tests.log`.
`target/native-dungeon-return-receipt` matches all54,279 receipt-driven
frames (61.99s). No full1,581,079-frame gate was repeated for this binary.

Source `target/native-dungeon-return-source/return-phase-summary.json`
proves final port-clear bus V1/C1014, PLP at1020, CLI at1090,
caller LDA at1148, STA instruction at1164, and RTS at1194. The $4200
bus access is1188, 174 CPU clocks after the final clear. A regression
checks this active-display return and that forecasting does not retire
the live receiver. Native restores at1190, retaining the unresolved
two-clock command phase rather than compensating for it.
Native/source main counter, latch, and spotlight radius agree across
54,043..54,047; completed OAM also agrees on54,043/45/47.

### Next: the following overworld upload's interrupted common suffix

Source `target/native-54276-source`, resumed from the valid paired53500
checkpoint, remains video exact through54,278. Frame/NMI trace decoded
to `/tmp/native-54276-source.jsonl` shows the upload returning in54,250,
then NMI interrupts `$00:864f` inside sprite preparation at V225/C12.
Source reaches the main wait in54,251 and starts the next main iteration
in54,252. The native return forecast is host54,251, V220/C1010; its
current return branch completes the common suffix atomically. Trace its
remaining CPU work and reuse the typed partial sprite-preparation owner
if the measured suffix crosses NMI. Do not add a host delay based on the
return scanline or patch the later opening iris.
`target/native-54276-diagnostic/caller-state-comparison.json` confirms
the premature native latch clear at54,250 and counter increment at54,251.
That one-host lead survives overlay/loading returns into the visible iris.
Both native/source OAM buffers are identical throughout54,249..54,272;
the earlier scheduling difference precedes the eventual OBJ divergence.

### Previous native frontier — 54,045 (video)

The dungeon upload now queues its command at the caller's measured STA
bus access. The pre-dungeon CPU probe continues through the conditional
caller, accounts for masked host boundaries, and records only the command
host and raster position. It copies no shadow RAM into gameplay. The
receiver uses the dungeon entry's 386 CPU clocks to its first ready read,
including refresh, rather than the overworld entry's 364 clocks. A source
regression checks the first three read timestamps (660, 666, 718 from a
V118/C234 command). No frame-specific offset or hold-count change was added.

`target/native-dungeon-upload-command` proves exact native A/V through
54,044, first video mismatch 54,045 with audio still exact (78.40s).
Binary SHA: `30bb6f5783119485716163f3bb7d9857f10390e3151fe0de5c889d45c1914424`.
`target/native-dungeon-command-receipt` also matches all 54,047 receipt-driven
frames (61.99s). The prior 1,581,079-frame proof belongs to the older frozen
binary; this batch has not repeated that full run.
All 1,791 library tests pass, 3 ignored (23.75s):
`/tmp/native-dungeon-upload-command-lib-tests.log`.
The command remains two master cycles late against source (V118/C236
versus C234); that inherited phase error has not been compensated away.

### Next: derive the dungeon upload return from receiver completion

`target/native-54045-diagnostic` and `target/native-54043-source2` show
native's main counter one iteration ahead by comparison frame 54,043
(76 versus 75), then 77 versus 76 at 54,044 and 54,045. Source finishes
the upload in active scanout at 54,043, reaches the main wait, and only
starts the landing iteration in 54,044 (interrupted in Link OAM).
The native scheduler still uses `PRE_DUNGEON_SONG_BANK_TRANSFER_NMI_SLICES`
and `lane_finish_pre_dungeon_song_bank_transfer`, which can start a
successor iteration on its estimated terminal host. Trace this boundary
before changing spotlight/OAM publication. Replace the fixed estimate
with actual receiver completion and the source caller suffix; do not
increase the count or freeze a display generation to hide the mismatch.
Source presentation evidence: `target/native-54045-source-presented`,
paired diagnostic `target/native-54045-source` (video exact through 54,046).
Native presentation: `target/native-54045-diagnostic-presented`.

### Previous native frontier — 54,043 (audio)

Ordinary overworld CPU probes now continue from their common suffix into
the main wait loop and retain the actual CPU registers and timing budget
at the next NMI. The next ordinary overworld or initial iris probe consumes
that phase instead of reconstructing its PC and flags. Only CPU timing
crosses the boundary; no shadow RAM is copied into gameplay. The retained
phase expires once its host has passed, so dialogue/scrolling work cannot
leave a stale snapshot for a later overworld return. A regression covers
future, current, and expired host ownership.

The overworld probe's fallback NMI seed now also distinguishes leading
from trailing NMI when selecting physical field parity. A trailing NMI
belongs to the following host; retained budgets preserve this parity and
the existing refresh/HDMA timeline across ordinary iterations.

`target/native-overworld-wait-phase3` /
`/tmp/native-overworld-wait-phase3.log` proves native exact A/V through54,042,
same first audio mismatch54,043 (76.04s while tests compiled). Binary SHA:
`7754149827779b98f796a7a6ccbff5b61b7a0c5ff3e87ef87b135439a780acb0`.
All1,790 library tests pass,3 ignored (23.56s):
`/tmp/native-overworld-wait-phase3-lib-tests.log`.

### Previous command diagnosis (wiring now implemented above)

The iris now inherits `$00:8034` at V225/C30 for host53,925, versus source
comparison53,924 `$00:8034` at C28. It reaches `$02:9982` at V255/C510,
versus source508. Thus the PC/flag owner is correct but the inherited
timeline remains2 cycles late. Pre-dungeon entry remains V248/C1200
(source1198), loader return V117/C902 (source900), and command-after
V118/C242 (source240). There is still no positioned dungeon command:
`begin_runtime_song_bank_transfer` queues bank1 at the audio window end.
Connect the measured caller position to the existing receiver protocol
using the dungeon's386-cycle first-read path, rather than an audio offset
or a different fixed host count. Account for the actual6-cycle final bus
access of the command STA; do not subtract the unresolved2-cycle error.
The command position is not yet source-exact. Trace the last fallback
phase after an intervening caller to resolve the inherited difference.

The iris-only prototype (`native-iris-wait-phase`) still inherited the
synthetic predecessor and did not solve the phase. The first continuous
prototype (`native-overworld-wait-phase`) stopped on a stale host44254
after dialogue returned at44592; the expiry fix removes that invalid reuse.
`native-overworld-wait-phase2` retained the wrong field parity. Use phase3
for current evidence. No full receipt gate was repeated;100k native remains
pending.

### Previous physical field-parity fix

The iris and pre-dungeon CPU probes now select physical field parity from
their entry raster position. A comparison host spans V225 through the next
V225; parity flips at V0 inside that host. These probes had used the
active-display field even for their V248/V255 entries, putting the short
scanline240 in the wrong field. `native_cpu_field_timing_at_entry` handles
both entry phases. The saved Snes9x state
`target/native-source-pair-53500/oracle.state` proves CPU.V_Counter225 and
TIM.InterlaceField1. `snapshot.cpp` saves the actual `S9xInterlaceField()`;
the TIM block's first11 fields are32-bit and the next byte is this flag.

Before the correction, matched loader NMI PCs alternated2/6 cycles late.
Afterward,52 of57 NMI PCs match source with a uniform2-cycle difference;
the other5 accept at neighboring instructions because that remaining
entry error straddles the NMI threshold. The loader's `$02:8350` return
improves from V117/C906 to C902 (source900), and command-after improves
from V118/C246 to C242 (source240). This removes the field error without
compensating the remaining caller phase.

Evidence: `target/native-upload-nmi-phase` (before correction,65.46s),
`target/native-upload-field-phase` (74.11s while tests compiled), and its
`loader-nmi-alignment.json`; source decode
`/tmp/native-upload-all-nmis-source.jsonl` uses raw run +53,500.
Native A/V still matches through54,042, same first audio mismatch54,043.
Binary SHA: `471ee30659fd9372d3b8937a571ca1a6440646a180faa6163aac0e350e74ca7f`.
All1,789 library tests pass,3 ignored (23.91s), including a regression that
requires consistent field lengths across V0 and the host's V225 boundary:
`/tmp/native-upload-field-phase-lib-tests.log`.

### Prior diagnosis: synthetic leading-NMI phase at iris entry

`target/native-iris-entry-phase-source` /
`/tmp/native-iris-entry-phase-source.jsonl` proves comparison53,924 enters
Module0F at `$02:9982` V255/C508; native entry host53,925 uses C510.
Source's leading NMI is at `$00:8034` V225/C28, followed by `$00:8051`
V251/C118. `module_cpu_entry_after_leading_nmi` instead seeds `$00:8036`
at the earliest acceptance boundary with a synthetic zero flag. Trace the
preceding caller's actual wait-loop phase and carry it forward; do not
replace this with a fixed28-cycle NMI or subtract2 from the result.
The recurring source entries53,927 V255/C528 and53,929 V253/C204 also
provide checks. This remaining inherited phase reaches pre-dungeon entry
at nativeV248/C1200 versus source1198. After correcting it, wire the
measured dungeon command's bus access into the existing transfer protocol,
with its386-cycle caller path. No full receipt gate was repeated.

### Previous native upload interleaving fix

Native song-bank transfers now interleave CPU handshake accesses at SPC
micro-operation boundaries even when their command timestamp is not yet
measured. This separates hardware port visibility from caller timestamp
ownership; receipt-driven transfers retain their existing scheduling.
A regression test uses `MOVW $f4,YA` to verify that an earlier CPU ready poll
cannot see the instruction's later output stores and publish a header early.
All1,788 library tests pass,3 ignored (25.32s):
`/tmp/native-upload-interleaving-lib-tests.log`.

This fixes intermediate port visibility but does **not** fix the current
audio divergence. `target/native-upload-interleaving` (65.28s) retains the
same first mismatch and audio hash. Its receiver `(PC,A,X,Y)` alignment
with source is unchanged, though some intermediate input-port counters
are no longer published early. Binary SHA:
`959e7139d50b8f1dd51647fae71076bc1efff4080fd65f31b783882bce6cb010`.

The additional development-only `ZELDA3_DEBUG_SONG_UPLOAD` probe continues
the isolated pre-dungeon CPU run from `$02:8350` to the conditional command
store. `target/native-upload-command-probe` /
`/tmp/native-upload-command-probe.log` (65.18s) reproduces the same exact
prefix with this instrumentation; binary SHA:
`b728f49ecc4e74b11f17e70e7a9d93faa1e60438b9e35d9804c97d346f152c1b`.
The full library suite preceded this diagnostic-only addition.

### Next: fix the command caller's CPU phase, without an offset

At native entry host53,963, both probe endpoints are V248/C1200. They
count57 loader NMI crossings and reach the instruction after the `$ff`
store at V118/C246. The actual unpositioned transfer begins on audio
host54,019 and currently queues its command at the audio window end.
An earlier dungeon upload is also covered by the generic probe:
entry host11,481 V248/C1188,58 crossings, command-after V197/C336,
unpositioned audio host11,538. State host and audio host differ by one.

Source `target/native-upload-caller-source` /
`/tmp/native-upload-caller-source.jsonl` (raw run +53,500) proves:
comparison53,962 `$00:8051` is V248/C1198; comparison54,019
`$02:8350` is V117/C900, `$02:9bff` V118/C210, and following
`$02:9c02` V118/C240. Thus the native probe is already2 master cycles
late at entry and6 late after the command instruction. The actual command
bus access is C234,6 before the following instruction. Do not subtract
12 from the probe as a correction; diagnose the caller/bus phase.
The source's first ready reads are V118/C660,666, then718,724:
the dungeon path takes386 CPU master cycles plus the crossed40-cycle
WRAM refresh from command to first read. It must not reuse the overworld
caller's364-cycle path. The measured command is not yet wired into native
dungeon playback. No full receipt run was repeated;100k native remains pending.

### Previous closing-iris fix

Closing-iris continuations retain their selected phase across table
completion, rather than replacing a measured CPU phase with a geometry
fallback. The native CPU plan also retains a second NMI inside Link's axis
loop: its completed subpixel/coordinate-store phase and the following
field's sprite-preparation completion. The existing partial movement
continuations now receive that native checkpoint, without replaying movement
or prematurely authoring Link OAM. Entry and recurring table resumes use
the same mechanism.

Source comparison53,925 ends after a second NMI at `$07:e3cb` (V225/C18),
after the Y low-coordinate store but before its high byte. The native probe
finds the same instruction at V225/C24. At53,924..53,929, the fixed native
counter, player bytes `$20..$31`, and OAM shadow `$800..$a20` agree with
source. The native software latch still reflects a different host phase at
53,926 and53,928; do not call these full-WRAM matches.

`target/native-iris-second-interrupt` /
`/tmp/native-iris-second-interrupt.log` proves native exact A/V through54,042;
first audio mismatch54,043, video still exact (69.65s while tests compiled).
Binary SHA:
`73771fad011ff9494cc76e330c1188bb2de7df32aa0871a60862fe966670768a`.
Library validation:1,786 passed in the full run (23.87s); the new test's
accidental whole-WRAM comparison was corrected to the OAM range and passed
separately. All1,787 tests are covered,3 ignored:
`/tmp/native-iris-second-interrupt-lib-tests.log`,
`/tmp/native-iris-second-interrupt-corrected-test.log`.
Tests retain a measured entry phase despite short geometry and verify that
the second native interrupt leaves Link OAM and sprite preparation pending.

### Next: song-bank upload completion

Source `target/native-54043-source2`, `/tmp/native-54043-source2.log`;
decode `/tmp/native-54043-source2.jsonl` uses raw run +53,500. Source frames
54,036..54,042 remain inside the APU upload loop `$00:88a4..88bb`, module7/
sub0f, counter75. Frame54,043 enters at `$00:88b6` V225/C18 and returns to
main wait `$00:8034` V225/C4. The next field begins landing iris work.
Trace the final upload handshake and native SPC scheduling before changing
any audio marker.

Initial upload evidence (before the interleaving fix above):
`target/native-54043-source-ports`, `/tmp/native-54043-source-ports.jsonl`
captures CPU/APU accesses from the valid53,500 pair. The actual `$ff`
command write is comparison54,019, PC`$02:9c02` (following the store),
V118/C234. Do not confuse later `$ff` transfer counters at PC`$00:88be`
with that command. Final clears in54,043 are V1/C924,954,984,1014;
there is no return NMI in that comparison field. Therefore the previous
overworld upload's missing return-NMI fix is not the explanation here.

`target/native-54043-upload-transport` reproduces the same first mismatch
(64.70s), with no timed dungeon command logged. At that revision, bank1 used
`begin_song_bank_transfer(..., None)`, whereas bank0's native path supplies
a measured command position and permits CPU/APU handshake interaction at
SPC micro-operation boundaries. The pre-dungeon timing probe stops at
`$02:8350`; its caller currently schedules the subsequent transfer with
`PRE_DUNGEON_SONG_BANK_TRANSFER_NMI_SLICES` (22). Replace missing timing
ownership using the caller/receiver protocol, not by adjusting this count.

Instruction evidence: `target/native-54043-upload-instructions` (65.25s)
and `target/native-54043-source-instructions`, both with DSP trace JSON for
54,042 and54,043. At54,042, native instruction0 matches source instruction9
for4,258 consecutive `(PC,A,X,Y)` tuples. At54,043, native instruction0
matches source instruction7 for1,243 tuples; driver return PC`$0a16`
occurs at native index626 versus source633. Relative to each trace's first
instruction, that return is2,500 versus2,528 APU cycles. These are relative
trace measurements, not proof of an absolute28-cycle clock offset. Establish
the command boundary and shared clock origin before changing scheduling.
No audio compensation, frame exception, or speculative runtime fix was added.

The attempted checkpoint at54,000 failed because the loader still held an
unserialized ROM-call continuation. Do not use `target/native-source-pair-54000`.
The valid source checkpoint remains `target/native-source-pair-53500`.
No full receipt gate was repeated;100k native remains pending.

## Previous native frontier — 53,926 (video)

Native overworld HUD inventory work now has a typed continuation across
NMI. The CPU probe records which conversion is interrupted, completed
value-load/call cycles, and elapsed conversion cycles. The translated HUD
commits completed digit groups, retains unfinished groups, and resumes only
the remaining conversion, HUD update flag, rain caller, and common sprite
preparation suffix. It does not rerun refill or player logic. Live receipt
playback never creates this native continuation.

Two source boundaries are covered by this batch: `$0d:f105` inside arrow
conversion near53,742, and `$0d:fc8b` before bomb conversion at53,763.
The C port's redundant backdrop writes no longer clear unfinished numeric
slots or the key label before their owning conversion returns. Completed
full HUD output and total instruction charges are unchanged.

`target/native-hud-entry` / `/tmp/native-hud-entry.log` proves exact native
A/V through53,925, first video mismatch53,926 with audio exact (75.47s while
tests compiled). Binary SHA:
`98d5e4c88f3f4972e68dc645d307ec636eec58da295da663c5d08e4df71bb455`.
Direct WRAM comparisons at53,742..53,744 and53,763..53,764 match source
counter/latch and the entire HUD buffer `$c700..$c880`, including the partial
field. Source sessions: `target/native-53745-source` and
`target/native-53763-source`; their decodes are `/tmp/native-53745-source.jsonl`
(raw +52,000) and `/tmp/native-53763-source.jsonl` (raw +53,500).

All1,785 library tests pass,3 ignored (23.99s):
`/tmp/native-hud-entry-lib-tests.log`.
Regression tests suspend all four conversions both before the JSR and
inside the conversion, check the source-derived total cycle cost, and
ensure resumption cannot rewrite already completed digits or erase pending
ones. No full receipt gate was repeated. The100k native target is pending.
The latest genuine paired source checkpoint is `target/native-source-pair-53500`.

### Next: closing iris entry

Source `target/native-53926-source[-presented]` resumes53,500 and matches
its enabled video lane through53,940. Decode `/tmp/native-53926-source.jsonl`
uses raw run +53,500. Module0F entry is visible at53,924; source NMI PCs
are `$00:f4e8` at53,925 (counter57/latch1, sub0), `$07:e3cb` at53,926
(counter57/latch1, sub1), then main wait at53,927 (counter57/latch0).
Native diagnostic output is `target/native-53926-diagnostic[-presented]`,
`/tmp/native-53926-diagnostic.log`; inspect process state before restarting
if this session is still running. Presented host53,927 owns comparison53,926.

## Previous native frontier — 53,745 (random-call timing)

Interrupted native overworld sprite preparation now retains the scroll
registers installed by its leading NMI for the current field. The main
slice's new software mirrors and the trailing handler's writes belong to
the following field. Existing typed scroll provenance handles both an
already captured display and an imminent capture; Live receipts are excluded.

`target/native-prep-field-scroll` / `/tmp/native-prep-field-scroll.log`
passes the previous52,448 video failure, then stops when sword-charge
sparkle creation requests random during53,745 rather than source53,746
(`ancilla.rs:657`). The panic leaves partial final ledger lines. Comparing
all complete candidate records directly against the pinned full oracle
proves exact video and audio for53,742 contiguous frames,0..53,741:
`target/native-prep-field-scroll/completed-prefix-validation.json`.
Do not claim rendered parity through the later panic frame.
Binary SHA:
`7487e9b7f1d260bf458dae92d3287870075facde1a5d2d0f6e0a2dc62fb4f6c5`.
The cold run took72.68s while library tests compiled. All1,783 library
tests pass,3 ignored (24.50s):
`/tmp/native-prep-field-scroll-lib-tests.log`. The new regression verifies
that the interrupt retains its scroll for one capture and leaves live
registers and the following capture independent.

### Next: interrupted HUD decimal conversion

Source `target/native-53745-source` resumed the genuine52,000 checkpoint,
matched its enabled video lane through53,755, and saved a new paired source
checkpoint at `target/native-source-pair-53500`. Decode:
`/tmp/native-53745-source.jsonl` (raw run +52,000). Source comparison53,742
returns inside NMI with counter133/latch1;53,743 finishes with counter133/
latch0. The interrupted PC is `$0d:f105` in `Hud_IntToDecimal`, V225/C20.
Native WRAM in `target/native-53745-diagnostic` already clears the latch at
53,742, then advances to counter134 at53,743 and135 at53,744.

The native CPU probe logs `host=53743 entry=None pc=0df10f` V225/C24,
with no packing/source/pointer progress. Its current return type only
represents sprite preparation, so a HUD interruption returns None. Check
the probe host phase against comparison frames before interpreting that
label. Extend the actual HUD/main continuation and preserve its partial
work; do not delay or offset the later random call. The random panic at
53,745 is downstream of the earlier missing held iteration.
`/tmp/native-53745-diagnostic.log` completed in64.91s.

No full receipt gate was repeated.

## Previous native frontier — 52,448 (video)

Recurring native dungeon-iris holds now capture resident OAM and Link VRAM
at the interrupt boundary and carry that memory into the pending display
publication. A retained older snapshot can no longer undo the preceding
full NMI's upload. The initial pre-spotlight retirement keeps its existing
owner, and Live receipt playback never creates this native memory record.

`target/native-held-goal-provenance` / `/tmp/native-held-goal-provenance.log`
proved that at host50,756 the resident PPU and host-boundary VRAM already
held the correct new generation (OAM c678/ca7d, VRAM word4020=0040), while the
selected snapshot still held c778/cb7d and0000. The fixed candidate matches
source VRAM and OAM exactly at that boundary. The diagnostic remains under
`ZELDA3_DEBUG_DISPLAY_OAM_FRAME` as `interrupted_obj_publication`.

Native cached A/V is exact through52,447; video differs at52,448 with audio
exact: `target/native-held-resident-obj`,
`/tmp/native-held-resident-obj.log` (74.68s while tests compiled). Binary SHA:
`ac9c5bff392366f9e68dc35ada89b05f73a9514f0f0c3ebff47e01ec8498aaa7`.
All1,782 engine tests pass,3 ignored (24.30s):
`/tmp/native-held-resident-obj-lib-tests.log`. The new regression advances
live hardware after the interrupt, then checks that two display captures
retain the interrupt's memory and leave live hardware unchanged.

### Next: overworld sprite-preparation interruption

Source: `target/native-52448-source[-presented]`, decode
`/tmp/native-52448-source.jsonl` (raw run +48,481). A genuine paired source
checkpoint is now available at `target/native-source-pair-52000`; use this
for the next short source trace. Native:
`target/native-52448-diagnostic[-presented]`,
`/tmp/native-52448-diagnostic.log`.

Source comparison52,448 accepts NMI at `$00:8768`, V225/C34, inside the
sprite-preparation pointer tail, counter186/latch1. Both engines retain that
counter for52,449 before advancing again. OAM, CGRAM, all window rows, and
WRAM source words0ac0..0aea match. Presented host52,449 differs in34 bytes
of Link VRAM only. Raw Link-page hash prefixes show native one generation
ahead: source hosts52,447..52,450 are `faef33481f00`, `faef33481f00`,
`e15840dba043`, `fb7640d25089`; native has `faef33481f00`, `e15840dba043`,
`fb7640d25089`, `fb7640d25089`. The earlier raw difference is not itself a
video failure because decoded-cache provenance can retain another generation.
The native CPU-packing probe (`target/native-52448-packing`,
`/tmp/native-52448-packing.log`) reaches the same `$00:8768` instruction at
V225/C50, with300 pointer-tail cycles and three completed pointer words.
The raw Link VRAM discrepancy was misleading: both decoded Link caches
already have `fb7640d25089` at presented host52,449. Retaining resident cache
in `target/native-prep-field-cache` leaves the exact same video mismatch;
that experiment was removed.

The rendered BG scroll differs: source host52,449 has
`[(427,2155),(164,2239)]`, native has `[(427,2154),(165,2238)]`.
The latter belongs to source host52,450. Window rows agree, but scroll rows
do not. The next candidate retains the leading handler's scroll registers
for the interrupted preparation field, before the trailing handler writes
the following field's registers.

No full or repeated receipt gate was run for this native-only increment.
The100k native target remains pending.

## Previous native frontier — 50,755 (video)

Native spotlight plans now retain the HDMA rows consumed before their first
measured NMI. A field-local `NativeSpotlightFieldScanout` keeps those rows
with their window controls, independent of the later CPU table/register
generation. It resets at every host entry and is never populated by the
Live receipt owner. Rendering composes it into the outgoing surface and
restores the live state afterward.

Two source-backed fixes share this mechanism:

- Closing goals retain the visible prefix and brightness through the first
  direct INIDISP store at `$00:f3e5`. Its measured output row owns blanking;
  the later EnableForceBlank call and cleared window mirrors cannot erase
  already scanned rows. This fixes49,036 without a fixed row exception.
- Recurring Module10 opening calls retain the first field's actual HDMA
  consumption, including races with the working-to-hardware table copy.
  At49,129 source's final three rows use the newly copied table while native
  previously retained the old table for the whole field. Those rows now
  match exactly. Initial opening entries retain their existing publication
  path; they can begin mid-field after a loader, outside a full row history.

Native cached A/V is exact through50,754, with a video-only mismatch at50,755:
`target/native-iris-first-field`, `/tmp/native-iris-first-field.log` (72.72s
while library tests compiled). Binary SHA-256:
`b4ddb4e97dfa9bf3292a0e526ddb2a3473807e79720ce30ea2cc34644bb3bb5d`.
All1,781 engine tests pass (3 ignored,24.77s):
`/tmp/native-iris-first-field-lib-tests.log`.
The intermediate closing-only binary passed49,128, then exposed49,129;
`target/native-terminal-field` (70.80s) and its1,781 passing library tests
in `/tmp/native-terminal-field-lib-tests.log` (24.18s).

Opening evidence: `target/native-49129-source[-presented]`,
`target/native-49129-diagnostic[-presented]`, and
`target/native-iris-first-field-presented/49130-*`. VRAM, OAM, and CGRAM
already agreed; the new candidate removes all three window-row differences.
Direct source PC trace: `target/native-49129-source-entry`, decoded to
`/tmp/native-49129-source-pcs.jsonl`. Source49,129 enters `$00:8051` at
V248/C1140 and copies224 words from V192/C200 through V221/C486.

Next boundary: `target/native-50755-source[-presented]`, resumed from paired
pre-frame48,481; decode `/tmp/native-50755-source.jsonl`. Source is finishing
a Module7/sub0f opening goal, reaching radius126 at50,755 and returning to
Module7/sub0 at50,756. Native diagnostic:
`target/native-50755-diagnostic[-presented]`,
`/tmp/native-50755-diagnostic.log` (61.70s). At presented host50,756, all
window rows and CGRAM agree. VRAM differs in116 bytes of Link's page and
OAM in just bytes0x199/0x1ad. Main/submodule, counter, and latch agree at
comparison50,755; the goal state is not a whole iteration early or late.

The key provenance finding is a one-field **reversion**: both engines present
the new Link/OAM generation at host50,755, but native goes back to its older
host50,754 generation at host50,756, then recovers at50,757. Source keeps the
new generation across all three fields. Source OAM bytes are c6/ca while the
reverted native bytes are c7/cb. Link-page hash prefix changes from
`ae3444d3aa` to `abd0567c97`; native alone switches back for the goal hold.
Native50,756 selects `RetainImmutableCapturedPpu`,
`RetainCapturedBeforeNmi`, and Link `HostBoundaryBeforeMain`. Investigate
that old capture's selection across the held goal NMI; do not change window
timing or delay graphics which were already correctly presented.

No full or repeated receipt gate has been run for this native-only batch;
the100k native target remains pending.

## Previous native frontier — 49,036 (video)

The pre-dungeon CPU measurement now stops at `$02:8350`, after both
Sprite_ResetAll and Dungeon_ResetSprites. It previously stopped at
`$02:834c`, omitting the second reset's NMI crossings before module7 was
published. The conditional song-bank transfer remains independently owned.

Source and native enter the load together at comparison48,481. Both reach
`$02:834c` after56 crossings, source V213/C410 versus native V213/C406.
Dungeon_ResetSprites then crosses another NMI: source reaches `$02:8350`
at comparison48,538 V242/C390, native V242/C382 after57 crossings.
Main/submodule, latch, brightness, and counter now match48,536-48,542.
Earlier measured loads retain their prior58/57/57 crossing counts.
This is a source-backed measurement extension, not a one-frame offset.

Evidence: `target/native-48541-loader` and
`target/native-48541-source-loader`; reusable paired pre-frame checkpoint
`target/native-loader-source-pair-48481`. The short source reset-return
trace is `target/native-48541-source-reset-return` with decode
`/tmp/native-48541-source-reset-return.jsonl` (raw run +48,481).

Native cached A/V now matches through49,035; video differs at49,036,
audio exact: `target/native-pre-dungeon-reset-native` (59.34s),
`/tmp/native-pre-dungeon-reset-native.log`. Binary SHA-256:
`5a9bdc1d8d863d76926a49802a30c3bbcdb8e86b2f7acbacb23d5aa7e95b036f`.
Build and native comparison validate this native-only probe change; the
prior50,000 receipt and1,781-test evidence below belongs to the preceding
binary. No repeated receipt/full gate was run for this measurement change.

### Next boundary: closing-goal scanout at49,036

`target/native-49036-source[-presented]` and
`target/native-49036-diagnostic[-presented]` isolate the final closing-wipe
field. Actual CPU main/submodule, counter, and brightness agree as Module0F
switches to Module8/sub0. Presented host49,037 has identical VRAM, OAM, and
CGRAM, but source retains brightness15 and windowsel330333 until force blank
at output row221. Native instead publishes brightness0, a whole-field blank,
windowsel0, and cleared screen-window masks. This is a display-generation
failure; changing only the blank row would still leave the wrong windows.

The direct PPU trace `target/native-49036-source-blank`, decoded to
`/tmp/native-49036-source-blank.jsonl`, establishes two INIDISP stores:
IrisSpotlight_ConfigureTable's store ending at `$00:f3e5` occurs at V221/C1044;
EnableForceBlank's later store ending at `$00:8942` occurs at V222/C426.
Both select output row221, but the iris store owns the first blanking event.
Raw run555 is comparison49,036 (paired pre-frame48,481). The source changes
window mirrors later at V223, after blanking has begun.

Use `ZELDA3_DEBUG_SPOTLIGHT_ENVELOPE` for `[SPOTLIGHT-BLANK]` and
`[SPOTLIGHT-FIRST-NMI]` diagnostics: these preserve the direct-store raster
and unnormalized interruption PC. The ordinary `[SPOTLIGHT-PLAN]` PC0 is
normalization of a non-table interruption, not the actual CPU address.
The next fix must retain the visible field's measured window rows and
register generation through the first direct blanking store. Do not add a
constant row221 exception or reuse the map-fade caller's pending state.

The native diagnostic `target/native-closing-goal-phase` confirms the first
store at V221/C1254 before the first measured NMI; its output row agrees
with source despite the210-cycle residual. The later `$00:8942` store is
V222/C634 and would incorrectly choose row222 if used alone. The first
interruption is `$0d:a1d8` at V225/C24, normalized to PC0 in the old log.
The plan records `active_window_words` only after that first NMI and
`following_window_words` after the second: both are already all blank.
The terminal visible field precedes both arrays. Preserve its own HDMA
history and first blanking event rather than substituting either array.

The diagnostic-only build retains the same49,036 video frontier and exact
audio (59.71s); `/tmp/native-closing-goal-phase.log`. Binary SHA-256:
`f473261fc70a4c91600bbd14d371d25b9e4131fe432e7dc5e9c8302f8a9346de`.
No gameplay behavior changed in this diagnostic increment, and no receipt
or full-route verification was repeated for its merge.

## Previous native frontier — 48,541 (video)

Fresh Module0E dialogue rendering now uses a measured message-loop entry.
Before the leading NMI, the existing isolated CPU probe runs from main wait
through the current sprite/module prefix to `$0e:c984`. Its one-shot
`native_dialogue_fresh_cpu_entry` supplies the CPU budget after refresh/HDMA
stalls. This replaces the fixed-span fallback for these entries; other
entry paths retain their existing budgets, and held glyphs resume their
remaining work. The measured-entry helper supports a vblank entry across
the field wrap and remains disabled under receipt ownership.

At comparison 48,101, source enters at V81/C162; native host 48,102
measures V81/C158. Previously one unpriced sprite call rejected the ledger
prefix and selected a 224,608-cycle fallback budget, starting an extra
glyph too early. Native now performs the final line click after the held
NMI instead of queuing zero for comparison 48,110. No audio marker changed.
Evidence: `target/native-48111-source-glyph`, decode
`/tmp/native-48111-source-glyph.jsonl`, and pre-fix native
`target/native-48111-glyph` / `/tmp/native-48111-glyph.log`.

The entry measurement exposed a second root cause at 14,208: an indoor
held glyph re-entered `Dungeon_PushBlock_Handler` through the translated
Module0E dispatcher. Its completed prefix charged another 184 cycles on
each held field. The call now runs only on a fresh iteration. Source
resumes directly inside VWF; no caller-return threshold or offset changed.
Source entry V26/C34 agrees with native V26/C10. Source completes its
sprite preparation and latch clear at comparison 14,207 before NMI.
Source evidence: `target/native-14208-source-entry`,
`target/native-14208-source-suffix-full`, and the reusable genuine paired
pre-frame checkpoint `target/native-dialogue-source-pair-14000`.

Combined native A/V is exact through 48,540, then video differs at 48,541
with audio exact: `target/native-dialogue-entry-prefix-native` (64.91s),
`/tmp/native-dialogue-entry-prefix-native.log`. Binary SHA-256:
`5b74459a51d52772ed85f8632c6fdbd38fa99b504d5240d36fd243ae0194bdfc`.
Engine suite: 1,781 passed, 3 ignored (24.20s),
`/tmp/native-dialogue-entry-prefix-lib-tests.log`.
Receipt-driven A/V: all 50,000 frames exact on this binary,
`target/native-dialogue-entry-prefix-receipt` (58.48s),
`/tmp/native-dialogue-entry-prefix-receipt.log`. The full 1,581,079-frame
receipt proof still belongs to `d7d92a85`; no full gate was repeated.

Next: a dungeon entrance spotlight (`07/0f`). Source
`target/native-48541-source[-presented]` passes video through 48,555
(audio disabled), resumed from pre-frame 47,200. Raw runs add 47,200.
Comparisons 48,539/48,541/48,543 end held inside LinkOam at `$0d:a49b`,
`$0d:a3f9`, `$0d:a412`; the alternating fields complete the caller.
Compare current native state and presented windows/graphics before carrying
forward the earlier opening-wipe diagnosis.

Current capture `target/native-48541-diagnostic[-presented]` shows the
earlier cause: native completes Module6 at comparison 48,537 (07/0f,
latch clear, counter63), source does so at 48,538. Native begins the
spotlight at 48,538/counter64 versus source 48,539. The resulting
held/completed fields remain one field apart. At presented host48,542,
VRAM/OAM/CGRAM match despite differing video, but this is downstream of
the CPU phase error; do not patch windows first.

Source loader-entry decode `/tmp/native-48541-source-loader-entry.jsonl`:
48,478 changes to Module6/counter62 but remains held; 48,479 is inside
`$09:c48a`, 48,480 reaches main wait/counter62, and 48,481 starts the
next Module6 iteration/counter63. Native closing-plan diagnostics are in
`/tmp/native-48541-diagnostic.log`; final radius7 plan is host48,479.
Next capture should enable `ZELDA3_DEBUG_DUNGEON_CPU_SCHEDULE=1` and
WRAM48,475-48,485 as well as the loader return. Compare the measured
pre-dungeon entry/crossing count with source before changing either.
`pre_dungeon_load_nmi_slices_at` stops at `$02:834c`, after Sprite_ResetAll
and before the separately owned song-bank transfer.

## Previous native frontier — 48,111 (audio)

Sprite preparation now retains instruction-boundary progress through the
fourteen graphics source words at `$865c-$86de`. The existing native
overworld CPU probe detected the NMI but discarded this interval between
extended-OAM packing and the pointer tail. Source comparison 47,974 stops
at `$8673` after writing body top/bottom and head top; native previously
finished the iteration, advancing the next field early.

`SpritePreparationProgress::SourceWords` carries the measured elapsed
cycles and completed word count. Native publishes only that prefix before
NMI, then the remaining source words, animation updates, and pointer tail
on return. Completed stores are not replayed; countdowns run once. The
typed display bridge publishes each word without bypassing native state.
The probe measures three words / 298 clocks and accepts at `$8673`,
V225/C46, versus source V225/C36; this is not a claim of exact CPU clocks.

Native cached A/V is exact through 48,110; comparison 48,111 has matching
video and differing audio. Evidence: `target/native-source-words-native`,
`/tmp/native-source-words-native.log` (61.21s). Binary SHA-256:
`496eb907478e67208a315fccd854c76f3065765a249bff03cd3b23dce30d6c62`.
Engine suite: 1,781 passed, 3 ignored (24.08s), including a regression
for deferred countdowns, retained stores, and total-cycle/state equivalence:
`/tmp/native-source-words-lib-tests.log`.

The full receipt proof remains the older `d7d92a85` binary below; no full
gate was repeated for this merge, per user instruction. Continue batching
native fixes toward 100,000. Next source capture `target/native-48111-source`
passes enabled video through 48,115 (audio disabled), resumed from the
genuine paired pre-frame 47,200 checkpoint. Its decode
`/tmp/native-48111-source.jsonl` uses raw run +47,200. The next audio
failure occurs during dialogue rendering; investigate fresh timing evidence.

Next-front diagnosis: `target/native-48111-diagnostic` captures actual WRAM;
main/submodule/latch/frame counter agree through 48,105-48,114. Native
clears `$012f` on dialogue returns 48,109 and 48,113 while source retains
12. `target/native-48111-dsp` traces native SPC instructions and DSP writes
48,108-48,111. At comparison 48,110 native changes SPC input port 3 from
12 to zero (cycle 819394052); source writes 12 throughout that field.
Native restores 12 at 48,111, but its SPC instruction path and DSP phases
already differ. Source port/DSP evidence:
`target/native-48111-source-audio-writes.jsonl`.

Do not compensate with an audio marker yet. Source click trace
`target/native-48111-source-click` / `/tmp/native-48111-source-click.jsonl`
shows an actual `$012f=12` store at `$0e:cacc`, comparison 48,109 V252/C18,
after the leading NMI cleared it. Native host 48,110 instead resumes
read position `$33`, glyph `$42`, Drawing with 32,992 clocks remaining,
then reaches read `$35` without a new click. Its marker trace is
`/tmp/native-48111-marker.log`: host 48,111 queues zero. Native read positions
are one ahead of source on interrupted fields 48,105-48,108 and
48,110-48,112; they converge at line returns. Determine the earliest
glyph-progress/timing cause before changing audio transport. Source WRAM
trace filters use the low offset `012f`, not the banked `7e:012f`.

## Previous native frontier — 47,975

Two native caller fixes extend exact A/V through 47,974:

1. WorldMap_FadeOut now consumes the measured CPU write boundary already
   used for DungMap_Backup. The shared `map_fade_cpu_blank_scanline` starts
   before the leading NMI with current input and runs through `$00:8942`;
   its one-shot field is `pending_map_force_blank_output_scanline`.
   Native measures V46/C280, output row 45, versus source V46/C308: the
   render-event bucket is exact, but do not claim exact CPU master cycles.
   Native presented host 47,334 now retains brightness 1 above row 45.
   This moves the video frontier from 47,333 to 47,621.
2. Save-menu initialization was absent from the native text-initialization
   scheduling eligibility (only dialogue submodule 2 was armed). Native
   Module0E/11 now uses the existing measured Text_Initialize plan, preserving
   its sprite caller while decompression is suspended. The save-menu flags
   and selection suffix have been split into
   `complete_save_menu_after_render_text`; the initializer's return executes
   that suffix before Module0E's scroll-register/common suffix. The ordinary
   save-menu call returns immediately while translated work is pending.
   Live initialization still uses its existing semantic progress authority.

The save-menu plan measures 4 prefix crossings and 1 caller crossing at
host 47,620. Native/source counters stay 14 through comparisons
47,619-47,624, then advance together. HUD flag/core-update state also match
through that interval and the following iterations. At comparison 47,618,
prior to initialization, the native latch/HUD flag still differ from source;
that is not claimed fixed by this batch. Enabled native A/V remains exact.

Evidence: `target/native-map-fade-native` (67.38s, first video 47,621),
`target/native-map-fade-presented`, `target/native-47621-diagnostic[-presented]`,
`target/native-47621-source[-presented]` (source video passes through 47,640,
audio disabled, resumed pre-frame 47,200), and
`target/native-save-init-native` (64.69s, first video 47,975, audio exact).
The latter includes corrected actual WRAM around the initialization.
CPU plan logs: `/tmp/native-map-fade-native.log`,
`/tmp/native-save-init-native.log`.

Accepted batch SHA-256:
`6cd76ad5ef07748423013e2ffdd4485b270fb438e465895b7545834320a966ee`.
Engine suite: 1,780 passed, 3 ignored (24.01s),
`/tmp/native-map-save-batch-final-lib-tests.log`.
The first suite run exposed a missing host-frame setup in the new test
fixture; the fixture now enters the host/main-loop phases before invoking
the scheduled save-menu call. Runtime code was unchanged by that correction.
Receipt-driven cached A/V: all 50,000 frames exact on this binary
(`target/native-map-save-batch-receipt`, 64.14s).
The full 1,581,079 receipt proof remains runtime `d7d92a85`; these are
native development CPU measurements, not finished ROM-less timing.
Continue batching toward 100,000 native before a full-route gate.

Next: investigate comparison 47,975 from source/native captures; do not
carry the prior HUD or save-menu diagnosis forward without new evidence.

## Previous native frontier — 47,333

`HandleStripes14` now programs DMA channel 1, leaving channel 0's PPU
register target intact. The bulk stripe implementation previously called
`program_dma0_ppu_target`, despite the ROM writing `$4310/$4311` and
triggering `$420b=2`. The next HUD DMA inherits channel 0's target, so
clobbering it silently manufactured a VRAM upload when source channel 0
still targeted OAM. This is shared hardware behavior, not a HUD-value or
frame exception. Copy stripes leave channel 1 in mode 1 at `$2118`;
fill stripes finish in fixed-source mode 0 at `$2119`, matching
`$00:92c2-$00:933d`. Channel 0 callers retain their existing wrapper.

Source comparison 47,233 ends its core updates with channel-0 OAM DMA,
then performs six channel-1 stripe copies at `$00:933d`. There are no
intervening source DMAs before comparison 47,237: `$00:8b87` consumes
HUD WRAM `$7e:c700` using channel 0, mode 0, B-bus `$04`. The ordinary
OAM DMA overwrites those temporary OAM bytes, then `$00:8d0d` uploads
text using channel 0 in VRAM mode. Thus source retains displayed rupee
1 while HUD WRAM already contains zero. Native previously used mode 1,
B-bus `$18`, prematurely publishing zero.

Evidence: source checkpoint `target/native-dialogue-source-pair-47200`;
fast source replays `target/native-47237-source-channel` and
`target/native-47237-source-dma`, both pass enabled video through 47,245
(audio disabled). Decodes `/tmp/native-47237-source-channel.jsonl` and
`/tmp/native-47237-source-dma.jsonl` use raw run +47,200.
Native pre-fix NMI diagnostic: `/tmp/native-47237-hud-pipe.log`.
Regression `stripe_channel_one_preserves_the_next_hud_dma_target` tests
both copy and fill followed by an inherited-target HUD transfer.

Accepted candidate SHA-256:
`aaaf44206425a9e3c0bfa4e1f78cf50412b9b06e5ad3d1867741a5a0c7b6113d`.
Native exact video/audio through 47,332; first video mismatch 47,333,
audio exact there (`target/native-stripe-channel-native`, 57.14s).
Engine suite: 1,779 passed, 3 ignored (24.53s),
`/tmp/native-stripe-channel-lib-tests.log`.
Receipt-driven cached video/audio: all 50,000 frames exact (58.52s),
`target/native-stripe-channel-receipt`, on the same binary.
The full 1,581,079-frame receipt proof still belongs to `d7d92a85`.

Next: comparison 47,333 is an ordinary main-loop return, immediately
before held work begins at 47,334. Capture actual native/source WRAM and
display domains; do not assume it is another HUD or dialogue failure.

Captured next-boundary evidence: `target/native-47333-diagnostic[-presented]`
and `target/native-47333-source[-presented]`. CPU module/submodule, latch,
brightness mirror, and counters match through 47,336. All presented VRAM,
OAM, and CGRAM match at engine hosts 47,332-47,336. At host 47,334 native
has brightness 1 with full forced blank; source has brightness 1 and a
forced-blank suffix beginning at presented row 45. Source PPU trace
`target/native-47333-source-blank` (video passes through 47,340; audio off)
shows comparison 47,333 restore `$2100=1` at `$00:8220`, V250/C1148, then
write `$2100=$80` at `$00:8942`, V46/C308. Decode is
`/tmp/native-47333-source-blank.jsonl`, raw runs +47,200. This is a
mid-field main-thread force-blank write during Module0E/7, not a tile,
palette, or OAM divergence. Native needs the measured CPU write boundary;
do not hard-code row 45 or borrow another module's fade estimate.

## Previous native frontier — 47,237

The sprite item-receipt caller now retires at the main wait before NMI.
Its final held handler interrupts decompression; the resumed sprite and
module suffix then update the HUD and return. The next open handler must
consume those operands before the next main iteration edits the HUD.
Previously only the ground-item caller restored this scheduler phase;
sprite and ancilla receipt continuations left it at the prior phase.
The fix is confined to native resumed sprite/ancilla callers after their
common suffix and existing item-graphics postlude. Live receipts retain
their own timing authority.

Source comparison 47,129 returns at `$00:8034`, V225/C8, with frame counter
97 and rupee goal/actual 0/99. Comparison 47,130 has counter 98 and actual
98 on both sides. The old native display nevertheless skipped 99: at
engine host 47,131, VRAM word `$606a` was `$2498`, source `$2499`. All other
VRAM bytes, OAM, and CGRAM matched. After the phase fix the displayed
99 -> 98 -> 97 sequence agrees with source, without changing the rupee
logic or load duration.

Evidence: `target/native-47130-source[-presented]` (source video passes
through 47,150, audio disabled), `target/native-47130-diagnostic[-presented]`,
and `target/native-item-return-phase-presented`. Source trace decode:
`/tmp/native-47130-source.jsonl`, raw runs +38,001.

Accepted binary SHA-256:
`d2ef7cf20e12afc46d8c31b5b952fa6f277e9de0ad3d41e69685cf753f5a9abc`.
Native video/audio exact through 47,236; first video mismatch 47,237,
audio exact there (`target/native-item-return-phase-native`, 57.47s).
Engine suite: 1,778 passed, 3 ignored (24.26s),
`/tmp/native-item-return-phase-lib-tests.log`. No new receipt replay;
the full 1,581,079 receipt proof remains runtime `d7d92a85`.

Next frontier follows a dialogue caller: source held/rendering at
47,233-47,236, common suffix completed at 47,236, open NMI and a fresh
main at 47,237, then rendering holds resume. Diagnose native/source
state and displayed text before altering dialogue duration.

Follow-up evidence narrows 47,237 to HUD publication, not text timing:
`target/native-47237-diagnostic[-presented]` and
`target/native-47237-source[-presented]`. Source video passes through
47,260 (audio disabled). Both CPU counters/latches match through 47,240;
rupee actual reaches zero on both at 47,233. Both HUD WRAM words at
`$c754` are `$2490`. Source presented VRAM word `$606a` remains `$2491`
through engine hosts 47,238-47,240, whereas native displays `$2490`.
That single byte is the entire VRAM difference at those hosts; OAM and
CGRAM match. At host 47,235 there are also 422 animated-page VRAM byte
differences, but enabled video was exact there. Do not confuse them with
the first visible failure.

A genuine source paired checkpoint is now available at
`target/native-dialogue-source-pair-47200` (pre-frame 47,200), saved by the
successful source replay. Use it for nearby source diagnostics; it is not
a native timing checkpoint. Next inspect the HUD/message DMA destination
and persistent channel-0 transfer plus display composition: a zero in
WRAM does not prove a completed hardware upload to the HUD destination.

## Previous native frontier — 47,130

The closing entry at comparison 41,244 had correct CPU counters/radius,
all HDMA window rows, OAM, and CGRAM, but 77 VRAM bytes in Link's OBJ page
already matched the following source field. Native host 41,245's full VRAM
matched source host 41,246; source hosts 41,244 and 41,245 retained the same
Link page. Evidence: `target/native-41244-diagnostic[-presented]` and
`target/native-41244-source[-presented]` (source video comparison passes
through 41,270, audio disabled).

The current-field graphics retention was conditional on a queued HDMA
receipt. At this entry the measured rows were already attached to the
active field, so generic capture bypassed that helper and published the
trailing Link upload. `retain_spotlight_entry_graphics_before_trailing_nmi`
now applies to both native entry-completion capture paths. HDMA routing
stays independent; the receipt-driven Live branch is unchanged. The
existing publication test now checks the capture's own OAM retention
instead of manually replacing that policy before resolving the plan.

An initial experiment changing only the existing helper had no effect and
was reverted before the complete fix. Its diagnostic log
`/tmp/native-entry-obj-owner-pipe.log` shows the bypassed path selecting
`ComposeLiveAfterNmi`, Link `LiveAfterMain/LiveAfterMain`, despite retained
OAM. Do not diagnose this as another CPU delay or table-row mismatch.

Accepted binary SHA-256:
`0fb57f18bea219f0ffc0b713cde2b2499c31c31634d9225d7df3e4eedb73ca7f`.
Native cached A/V is exact through 47,129; first video mismatch 47,130,
audio exact there (`target/native-entry-resident-native`, 60.88s).
Engine validation: 1,778 passed, 3 ignored (24.22s),
`/tmp/native-entry-resident-lib-tests.log`.
No additional receipt replay was run for this native-only publication fix.
The full 1,581,079-frame receipt proof remains the older runtime `d7d92a85`;
do not attribute it to this binary. Continue batching toward 100,000 native.
Next: capture source and native state/display at comparison 47,130 before
changing timing or graphics publication.

## Previous native frontier — 41,244

Two independent timing errors are fixed in this batch:

1. The closing entry's measured CPU plan proves that its shared sprite
   preparation suffix returns before the second NMI. The old geometry
   fallback nevertheless scheduled another `FinishSpotlightIteration` for
   long tables. `complete_dungeon_exit_spotlight_entry` now honors the
   measured completion instead. Source comparison 40,978 reaches `$00:85fc`
   at V37/C162 and returns at `$00:8036`; comparison 40,979 begins the next
   iteration. Native counters and radius now match throughout that interval.
   This advances native A/V from 40,980 to 41,078.
2. The pre-overworld screen build completed one host early: native switched
   to Module10 at comparison 41,074, while the source was still in Module8/2
   and returned at 41,075. The native path now measures the complete call
   from its leading NMI through the main-loop return and preserves every
   measured held crossing in `schedule_work`. The initial wait-loop visit
   to `$00:8036` must not terminate measurement before `$00:8051` is reached.
   Existing unmeasured/receipt execution keeps its prior path.

The earlier screen load measures 17 held NMIs (entry 4,868, return 4,885);
the latest measures 16 (entry 41,059, return 41,075), matching source
acceptance counts. Properties and overlays were already aligned at the
latest load; do not add a delay to either. The corrected screen return
also aligns Module10's counters/radius through 41,079. Its first source
entry is V31/C354 after the large NMI upload; that fact alone did not
justify changing the separate opening-entry estimate.

Final binary `cee84478d18a043c8f7539da19ed21c1b5b6ec0b4559f07bdbfd5ba23db4f9b0`:

- Native video/audio exact through 41,243; first video mismatch 41,244,
  audio still exact (`target/native-pre-overworld-entry-guard-native`, 51.21s).
- Engine suite: 1,778 passed, 3 ignored
  (`/tmp/native-close-screen-batch-lib-tests.log`, 24.19s).
- Receipt-driven cached A/V: all 50,000 frames exact on the same binary
  (`target/native-close-screen-batch-receipt`, 58.67s).

Source evidence: `target/native-40980-source` and
`target/native-41078-source`, both resumed from the 38,001 pair with enabled
video comparison passing (audio disabled). Raw run numbers need +38,001.
`target/native-pre-overworld-stage-diagnostic` captures actual WRAM across
the previously early loader; `target/native-pre-overworld-entry-guard-native`
captures its corrected state. Closing plan/publication summaries are under
`ZELDA3_DEBUG_SPOTLIGHT_ENVELOPE`; load counts/return rasters are under
`ZELDA3_DEBUG_DUNGEON_CPU_SCHEDULE`. Counts and semantic boundaries have
source proof; do not claim independently exact CPU return rasters.

The full 1,581,079-frame receipt proof still belongs to runtime `d7d92a85`.
These are development ROM CPU measurements, not completed ROM-less timing.
Continue batching native fixes toward 100,000 before another full-route run.

Next: another closing-entry return at comparison 41,244. The new source
capture `target/native-41244-source` passes enabled video through 41,270
(audio disabled); its presented dumps are in
`target/native-41244-source-presented`. Source 41,242 changes Module9/0 to
Module15/0, 41,243 begins the close (counter 167), and 41,244 returns at
`$00:8034`, V225/C8, with radius `$77` and latch clear. Native presented
dumps from this batch stop at engine host 41,200, so capture the new
native boundary before inferring its display or CPU cause.

## Previous native frontier — 40,980

Opening landing wipes now derive their displayed table generation from
the actual `$00:f3bb` (`STA $1B00,X`) copy stores and each row's HDMA read.
The native Module 7 CPU plan carries a per-row mask through its pending
and active iteration; both interrupted publication and caller-return
publication consume that result. Receipt-driven execution keeps its
existing authority. A single tail boundary is insufficient: a copy can
straddle the start of visible display and produce a different row pattern.

Pinned Snes9x store-completion checkpoints at `$00:f3be` prove the old
39,742 boundary: rows 219–223 complete at V220/C1286, V221/C90,
V221/C258, V221/C426, and V221/C634. Only rows 221–223 beat their reads.
At comparison 39,744 those stores complete at V224/C400, C608, C776,
C944, and C1154; none beat their reads. The source-backed regression is
`landing_copy_stores_race_their_own_hdma_rows`, with trace evidence in
`target/native-landing-copy-stores-source` (raw runs +38,001).

Final binary `ec5c742b3474952f1e7b02c6f9e6fe2ea7c9fab8590398a20fe56d8e3e5930cb`:

- Native video/audio exact through 40,979; first video mismatch 40,980,
  audio still exact (`target/native-landing-copy-final-native`, 54.76s).
- Engine suite: 1,776 passed, 3 ignored
  (`/tmp/native-landing-copy-final-lib-tests.log`, 24.25s).
- Receipt-driven cached A/V: all 50,000 frames exact on the same binary
  (`target/native-landing-copy-final-receipt`, 58.80s).

The previous 39,742 window mismatch is fixed. The previously recorded
463/386 VRAM byte differences at engine hosts 39,743/39,744 remain but
do not affect the exact compared image; do not claim those memory domains
are equal. This remains development ROM CPU measurement, not a completed
ROM-less timing implementation. The full 1,581,079-frame receipt proof
still belongs to runtime `d7d92a85`.

Next: closing Module0F spotlight publication at comparison 40,980.
`target/native-40980-source` resumes the 38,001 source pair and passes
enabled video through 41,000 (audio disabled). Compare its
`target/native-40980-source-presented` dumps with
`target/native-landing-copy-final-presented`; both have nearby actual
WRAM captures in their session directories.

At engine host 40,981, VRAM, OAM, CGRAM, and scroll match; 25 window rows
differ. Source windows match native host 40,982. At engine host 40,983,
27 window rows differ and source matches native host 40,984. Source
comparison 40,980 returns at `$00:8034`, V225/C0, radius `$70`, latch
clear, without accepting an NMI in that host. The preceding host accepts
two NMIs and returns inside `$00:f536` with radius `$77`, latch held.
Investigate the completed-field versus NMI-acceptance publication owner;
do not add a frame/room exception or offset the CPU clock.

## Previous native frontier — 39,742

Pre-dungeon loading now measures its room-dependent CPU workload instead
of always waiting 58 NMIs. The preceding Module0F CPU plan retains the
successor's entry at `$00:8051`; the loader follows that entry through
`Sprite_ResetAll` to `$02:834c`, before the independent song-bank transfer.
Both ends of the carried entry envelope must produce the same crossing
count. Entrances without that carried phase retain the existing fallback;
receipt-driven execution retains its existing authority.

The two exercised loads measure 58 and 57 held NMIs, matching counted
Snes9x acceptance events. Do not count host callbacks: callbacks with zero
or two acceptances made the previous 58/58 diagnosis incorrect. Also,
`$02:834b` is the stacked return address; RTL resumes at `$02:834c`.

Binary `558e53fcab9a5ee21051f1d5def23d73657b7eb350a1294e524a604e5fabe518`:

- Native video/audio exact through 39,741; first video mismatch 39,742,
  with audio still exact (`target/native-pre-dungeon-return-pc-native`,
  57.43 seconds). The earlier 11,538 audio boundary passes.
- Engine suite: 1,775 passed, 3 ignored
  (`/tmp/native-pre-dungeon-measured-lib-tests.log`, 24.37 seconds).
- Receipt-driven cached A/V: all 40,000 frames exact on the same binary
  (`target/native-pre-dungeon-measured-receipt`, 47.79 seconds).

Counts are source-checked; sub-frame entry timing is not yet exact. The
carried entries are V248/C1188 and V248/C1174, versus source C1170 and
C1178. Do not use an unexplained offset to reconcile them or claim that
the measured return rasters have independent source proof. This remains
a development ROM CPU measurement, not a completed ROM-less model.

The opening landing-wipe frontier now has a matching source capture:
`target/native-landing-wipe-source` resumes the 38,001 source pair and
passes enabled video comparison through 39,750 (audio disabled). Its
presented dumps are in `target/native-landing-wipe-source-presented`;
native baseline dumps are in `target/native-landing-wipe-presented`.
Raw trace run numbers need **+38,001**; presented engine hosts need **-1**
to obtain comparison frames.

At engine host 39,743 (comparison 39,742), OAM, CGRAM, and scroll match.
Only window rows 221–223 differ: native pairs are `(18,238)`, `(20,236)`,
`(20,236)`; source pairs are `(4,252)`, `(4,252)`, `(6,250)`.
VRAM also differs in 463 bytes: 233 at `$7600..$77ff`, 153 at
`$7800..$7fff`, and 77 above `$8000`. Each source range matches preceding
native hosts 39,739–39,742. At host 39,744 only the first two ranges still
differ; host 39,745 matches again. Do not conflate the window publication
and resident DMA discrepancies or assume fixing one resolves both.

The source copy-loop PC `$00:f3b7` on comparison 39,742 spans
V192/C214 through V221/C500 (last iteration, X446). The radius advances
`$3f->$46`, and the held NMI interrupts the landing's LinkOam at
`$0d:a416`, V225/C12. The next host reaches `$00:85fc` at V15/C340.
`spotlight_opening_projects_live_tail_before_hdma` currently cuts off at
post-build radius `$3f`, rejecting the source-visible tail of this copy.
Replace that radius heuristic with measured CPU-store/HDMA-read ownership;
do not just raise the cutoff or retune a raster constant. The next copy
(comparison 39,744) finishes its last loop iteration at V224/C1018, so
the same publication assumption does not apply to every larger radius.

Two disposable experiments were removed: measuring landing dispatcher
entry from the actual leading NMI, and additionally capturing its input
before NMI mutations. Both reproduced the identical first video mismatch
and hash at 39,742 with exact audio (`target/native-landing-measured-native`,
48.21s; `target/native-landing-pre-nmi-native`, 47.65s). They are not fixes.
Their builds overwrite `target/song-upload-build/parity/zelda3`; rebuild
the clean accepted source before using that path for further evidence.

Batch subsequent fixes before another full-route run. The old full
1,581,079-frame receipt proof still belongs to runtime `d7d92a85`, not
this batch.

## Previous native frontier — 39,727

After the requested local merge, the next source-backed fix separates
interrupted spotlight entry HDMA from the trailing NMI's graphics DMA.
The source entry return at `$00:f3b7` retains resident VRAM while channel 7
has already consumed the current field's rows. The former publication
selected 419 bytes of a future animated page at VRAM byte `$7800` and
77 bytes of future Link tiles at `$8040..$8278`. All 224 window bounds,
scroll rows, OAM, and CGRAM already agreed. The coarse dungeon-exit signal
also overrode explicit retained Link generations; it now follows the same
interrupted-entry ownership rule as OAM.

Binary `67f31d2df16e26d238ee5e7a1b5084336bd06e97074f6431eaab446d882843ec`
is native exact through frame 39,726. First video mismatch is 39,727;
audio remains exact there (`target/native-dma-owner-native`, 55.87 seconds).
The engine suite passes 1,774 tests with 3 ignored
(`/tmp/native-dma-owner-lib-tests.log`, 24.43 seconds).
Receipt-driven cached A/V matches all 40,000 frames on the same binary
(`target/native-dma-owner-receipt`, 47.76 seconds). The full-route proof
remains the older runtime proof described below; this is a focused regression.

Source evidence: `target/native-spotlight-source`, with video enabled,
and its presented dumps; domain comparison:
`target/native-entry-hdma-diagnostic/domain-comparison.json`.
The six-clock Module0F entry residue was not adjusted to obtain this fix.

Next boundary: receipts at 39,726 interrupt in LinkOam; 39,727 completes
a held NMI and the caller/common suffix without another NMI acceptance.
39,728 then accepts the next open NMI. Capture native/source display and
CPU return progress across that boundary before changing costs or owners.

## Pre-dungeon investigation after the 39,727 frontier

The next visible mismatch is the opening dungeon landing wipe (`07/0f`),
not the preceding Module0F close. At engine host 39,728, composed VRAM,
OAM, CGRAM and scroll agree; native keeps the window closed on rows
207..217 while source shows the first small opening. Native CPU state is
already one host late when Module_PreDungeon returns: comparison 39,723
still has native module 6, whereas the source has returned to `07/0f`.
Native returns on 39,724. Evidence is `target/native-link-return-diagnostic`
and `target/native-link-return-source` (resumed at 38,001, video enabled),
with their corresponding `*-presented` directories.

A blanket switch from `schedule_work(58)` to the scheduler's
`schedule_cpu_timed_work_from_current_main_iteration(58)` was tested and
removed. It exposed audio mismatch 11,538 and published native module 7
on comparison 11,537, before the original returned on 11,538. The
experimental artifacts are `target/native-pre-dungeon-crossing-native`
and `target/native-pre-dungeon-audio-diagnostic`; they are not the accepted
runtime. Main remains the 39,727-frontier runtime.

Counting actual `nmi` events (not host calls) gives **58 held NMIs for
the first load and 57 for the later load**. Some host calls contain zero
or two acceptances, so the earlier inference that both counts were 58
was incorrect. Their final handler spans also differ:

- First load starts 11,480. Its last held acceptance is `$09:c47f` at
  11,537 V225/C32. That host returns in the vector at V225/C94; the next
  host resumes at V227/C368 and enters the song-bank transfer. Module 7
  must not publish before 11,538.
- Later load starts 39,666. Host 39,722 returns at `$09:c47f`, V225/C4,
  before acceptance. Host 39,723 accepts at `$09:c480`, V225/C18,
  finishes the handler and caller, and reaches main wait at V225/C6.

Measure the actual room-loader workload and preserve its caller phase;
neither a universal decrement nor a room/bank exception is justified. The initial host also differs:
11,480 begins with an already accepted handler, whereas 39,666 begins
before the open NMI. Preserve the caller's CPU raster rather than inventing
an entry offset. The Module0F CPU plan already follows its final caller
through main wait but discards the successor-module entry phase; carrying
that phase into the pre-dungeon measurement is a promising next step,
not yet an implemented or verified fix.

The first-load source is `target/native-pre-dungeon-first-source`: cold,
video enabled through 11,542. It saved matched diagnostic Rust/oracle
states for 11,535..11,542, so future source audio probes can use explicit
`--resume-rust-state` / `--resume-oracle-state` with
`ZELDA3_LIVE_RNG_DIAGNOSTIC_RESUME=1` instead of another cold replay.
These are source/receipt states, not native SPC checkpoints. Decode source
traces from the resumed 38,001 run using relative run numbers; add 38,001
for route coordinates. The first-load cold trace uses route run numbers.

## Latest local merge — 2026-09-12

The user explicitly requested merging the accumulated native timing batch
into local main without repeating verification. This overrides the older
branch-only/full-gate-before-merge workflow below for this merge.

The batch includes SPC upload/caller-return timing, byte-level extended OAM
and final pointer-store continuations, measured map graphics NMI counts,
and spotlight entry/current-field HDMA publication. Existing checks on binary
`d96b3c27e3e42d2fb0aad1dabe9927d3c6d5d146d1238831b8efd02e179884df`:

- Engine library: 1,774 passed, 3 ignored (`/tmp/native-entry-hdma-lib-tests.log`).
- Receipt-driven cached A/V: all 40,000 frames exact
  (`target/native-entry-hdma-receipt`, 47.62 seconds).
- Native cached A/V: exact through frame 39,629; video first differs at
  39,630, with audio still exact (`target/native-entry-hdma-native`).

The last full 1,581,079-frame receipt A/V proof is for runtime `d7d92a85`
(`target/native-batch-full-d7d92a85`); it has not been repeated for this batch.
Do not attribute that full-route proof to the newly merged runtime.
The development binary remains in `target/song-upload-build/parity/zelda3`;
`target/parity/zelda3` remains the older fully checked binary.

Next: compare the current composed spotlight field at engine host 39,631
with `target/native-spotlight-source-presented`. The scoped HDMA publication
restored the earlier 11,443 boundary and changed the 39,630 image, but the
remaining cause is unresolved. Source entry is V255/C594 versus measured
V255/C600; do not introduce an unexplained six-clock adjustment.

## What the program is

The engine reproduces the game exactly, but only while it is driven by
per-host timing receipts captured from Snes9x. Those receipts tell it how
much CPU work each host frame performed. The goal is an engine that
derives the same timing itself, so the game runs bit-exact with no ROM and
no receipts present. The user's constraint is explicit and has been tested
against twice: **do not embed the ROM**. The ROM may be read at
development and test time only. Its data-dependent constants may be
extracted into the asset pack; its control flow must be modelled in Rust.

## The two gates

**Acceptance gate (never regress this).** A full-route cached comparison
against the pinned Snes9x oracle cache: every per-frame video and audio
hash must match.

```sh
./parity cached-av .git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18 \
  --output <run-dir> --paired-checkpoint-interval 20000
```

This drives the receipt path. It is currently exact over all 1,581,079
frames and is promoted in `routes/full_run/parity-frontier.json`.

**Development gate (the thing being moved).** The same route with no
receipts installed, so the engine must derive its own timing. The first
mismatching frame is the frontier.

```sh
ZELDA3_CACHED_AV_NATIVE_TIMING=1 ./parity cached-av <cache> \
  --binary target/alt/parity/zelda3 --frames 200000
```

Build that binary with `CARGO_TARGET_DIR=target/alt` so a frontier run can
never rebuild the binary a comparison is using.

History of the frontier: 8889 → 4660 → 2507 → 8716 → 7330 → 2507 → 8890 → 11444 → 4785 → 11444 → 20257 → 20262 → 23203 → 23206 → 23935 → **23945** (completed quadrant/audio batch).
It moves backwards whenever a newly exact cost exposes a wrong one
downstream; that is normal and not a regression of the acceptance gate.

The user explicitly reaffirmed this on 2026-09-11: an earlier native display
failure is acceptable when the change improves fidelity to the original.
Keep source-proven corrections with regression coverage even if they expose
an earlier native A/V frontier. Report CPU-model fidelity and native A/V
coverage separately. Receipt-driven acceptance must still pass; do not revert
a demonstrated timing correction solely to preserve the old native frame
number. This supersedes the overly conservative rejection in the spotlight
investigation recorded in `romless-exact-play.md`.

## Previous combined A/V and WRAM baseline

| | |
|---|---|
| branch | locally merged to `main`; three source-backed quadrant/audio fixes |
| promoted ledger | full route exact, four WRAM goldens and endpoint matched |
| library suite | 1,743 passing, 3 ignored; dev builds have zero warnings |
| native frontier | frame 23945, audio exact through that frontier |

Validated runtime commit: `e70152ae55c787ed3c53a86014dc805d604cd097`.
Binary SHA-256: `1245aeb75f851063dcb0610b06789f2efefb48f39e18c086b5a6005c199e6814`.
The full-route receipt is
`routes/full_run/receipts/quadrant-batch-full.manifest.json`.

The scroll-return milestone is complete. The native lane carries the scroll's
remaining CPU work, finishes a fitting return after the held NMI, and marks
the existing scheduler's main-wait phase so the next leading NMI publishes
text before another scroll starts. The caller's obsolete addition of
refresh/HDMA stall time to CPU headroom was also removed. See the scroll-return
history in `romless-exact-play.md` for the original timestamps, regression
test, exact validation and the remaining coarse pixel-copy limitation.

## Rules that are not negotiable

- **Never push.** The user pushes.
- Never embed the ROM. It was implemented once and reverted on request.
- Never regenerate frozen test fixtures.
- Run GPU comparisons serially. The binary takes an exclusive lock; do not
  run cargo GPU tests beside one, and never rebuild `target/parity/zelda3`
  while a comparison is running.
- Subagents must not run `cached-av` or any GPU comparison. They annotate
  and verify against the recorded shadow profiles; the lead measures.
- Never `git checkout <file>`; revert your own edits surgically.

## Current working batch: native 100k

The user requested a larger batch on 2026-09-11 targeting native exact A/V
of at least 100,000 frames. On 2026-09-12 the user explicitly allowed a full
parity check whenever needed, lifting the earlier prohibition before100k.
Continue batching; use full acceptance when the shared-path risk warrants it.
The full-parity-tested batch was merged to local `main` on2026-09-12 at
`edd069d420ca64e1abbe6442d3bd75018a42fad7`, with verification hooks skipped
at the user's explicit request after the full pass. No push. The new upload
experiment remains on `fix/native-song-upload`. Short receipt comparisons protect the shared path
while native timing advances; they are not native acceptance evidence.

Current native frontier: **37688, video-only**. This batch has corrected:

**Full receipt acceptance refreshed 2026-09-12.** Commit
`d7d92a8514455839b1c5fcd9adf8dcdafcbb5aa1` passes all **1,581,079**
frames from frame zero against the pinned cache, with exact video and audio,
contiguous coverage through1581078, and no RNG drift. Binary SHA-256:
`2fe6d9061b58cc048ca12ff2aed222337099c49e49314968b2b10082476cdb6e`.
Evidence: `target/native-batch-full-d7d92a85/manifest.json`, its full
`av_hashes.jsonl`, paired checkpoints every20,000 frames and `paired-final`;
log `/tmp/native-batch-full-d7d92a85.log`. The executable and clean source
revision stayed unchanged throughout the26.7-minute comparison. This proves
the accumulated native batch preserves receipt-driven full-route A/V;
it does not claim native100k, a new live-core run, or refreshed WRAM goldens.
The next upload candidate is isolated in `target/native-song-upload-worktree`
on `fix/native-song-upload`, with its own `target/song-upload-build` binary.
That uncommitted candidate is not covered by this full pass.

**Unmerged upload investigation (2026-09-12).** In the separate checkout,
native reaches **38732, video-only** (`target/song-upload-return-nmi-native`), versus
main's37688 video frontier. The existing SPC byte/ack owner now determines the
native caller's masked wait and same-host return, using a forecast of the
receiver rather than a fixed host count. Source return host37686 corresponds
to enginehost37687. The measured command plan must distinguish trailing from
leading NMI entry; pass that fact before `begin_trailing_nmi_receipts` opens a
write scope. Inspecting that scope afterward misclassifies trailing entry.
Native temporary timing fields are skipped by serialization to preserve the
existing receipt audio payload layout; native interrupted-call checkpointing
is not newly claimed.

The investigation also found a general timing defect in
`crates/snes/src/cpu_step.rs`: Snes9x `S9xOpcode_NMI/IRQ` prices its initial
opcode-fetch cycle with `CPU.MemSpeed`, not a fixed6. The candidate's timed
executor now charges62 for slow-ROM interrupts and60 for fast ROM; WAI wake
remains separate. All401 SNES tests pass,5 ignored. This removes82 clocks of
the104-clock command-prefix error across the leading plus40 Held NMIs.
The upload-specific main-wait seed now uses the source's `$8036`/zero-flag
busy-loop phase instead of synthetic WAI. Fresh source APUI bus traces correct
the earlier mixed-coordinate comparison: command bus V31/C832 precedes the
following instruction at C838. Source final port clears occur at
V251/C470,500,530,600; the caller restores $4200 at bus C774, before the
following instruction at C780. The candidate now matches command832 and
restore774. Its first ready-poll low read is364 master clocks after the
command, matching source1196, high1202 and failed-pair next low1254.
Timed uploads service host polls at SPC pseudo-op boundaries, preventing a
poll from seeing a later store in the same instruction. Receipt-era untimed
uploads retain their existing behavior. Four source-backed upload tests and
all1771 engine library tests pass,3 ignored; logs
`/tmp/song-upload-return-nmi-tests.log`, `/tmp/song-upload-return-nmi-lib-tests.log` and
`/tmp/song-upload-micro-snes-all.log`. The upload suite now has five tests.
Latest short receipt A/V:40,000 exact, `target/song-upload-return-nmi-receipt`.
Source: `/tmp/pre-overworld-upload-source.jsonl`; per-crossing native trace:
`/tmp/song-upload-native5.log` (before the interrupt-cost correction),
`/tmp/song-upload-native6.log` (after), and `native7` (busy-loop seed).
The earlier37749 candidate and receipt produced the same66 DSP register/value
writes, but every native timestamp was two APU cycles later. Source evidence in
`target/song-upload-source-ports` uses the exact pinned cache core binary;
its37749 audio hash matches the cache. Do not compensate DSP timestamps.

The first persistent receipt/source clock drift is now localized to34636,
well before this upload. `target/song-upload-clock-shift` contains two
instruction traces:34635 aligns4202 instructions with phase0;34636 aligns
1745 at phase0, then2486 at phase126. SPC `$08e8 MOV A,$00f4+X`, X=3,
reads0 in Rust and12 in Snes9x; their different branches reconverge at
$08a4 two cycles apart. Source NMI writes APUI03=12 at V225/C862.
`target/song-upload-click-transport` proves the receipt clock instead writes0
with `vwf_boundary_policy=0`, despite the preceding physical latch being12.
This run still has34,638 exact A/V frames, so internal clock fidelity and
rendered parity are distinct. **Native already publishes12 here**:
`target/song-upload-click-native` and the cached instruction capture in
`target/song-upload-native-source-alignment` match4241 source instructions
with phase0 and no timer-divider drift. Do not change native VWF ownership
to fix a receipt-only internal difference.

The actual native audio defect was the held NMI after upload return37686.
The transfer suppressed NMI audio for its entire host window, losing the
source's APUI01=5 publication at V252/C80 after its four port clears.
The candidate now queues that NMI at its real return: restore bus774,
remaining STA6 + RTS42, hardware NMI62, vector entry884. Its handler's
CPU work reaches APUI01/02/03 at V252/C80,174,236, with refresh priced
from the actual entry rather than the normal V225 entry. Command latches
are captured before `interrupt_nmi_audio_parts` consumes them; the delayed
audio queue is not the source of this immediate held-NMI publication.
`target/song-upload-return-nmi-native` then aligns4301 instructions at37686
and4205 at37749 with phase0 and no timer-divider drift. Audio remains exact
at the new video frontier38732. All fields remain native-only and skipped
by serialization; this candidate still needs its eventual full acceptance.

The next source receipt interrupts ordinary overworld sprite preparation:
38731 ends with `MainLoopInterrupted(SpritePreparation)`;38732 accepts a
Held NMI, continues the caller and finishes the common suffix. Native needs
the corresponding measured caller boundary rather than an early latch clear.
Cold diagnostic `target/native-overworld-prep-source2` proves entry to
`$0085fc` at38731 V219/C478; the host returns at V225/C4, PC `$00861f`,
X16/Y4, latch1. The next NMI accepts at `$008620`, V225/C18; the caller
finally reaches `$00805d` at V230/C840. This is inside the extended-OAM
packing group's second byte, after its first store to `$0a04`; existing
group-granularity helpers alone do not describe every committed byte.
Decoded trace: `/tmp/native-overworld-prep-source2.jsonl`. The source's
resumable paired checkpoint is
`target/native-overworld-rolling/frame-00038001`; a fixed capture at38000
was rejected inside a translated continuation, so use rolling captures.
`target/native-overworld-resume-check` successfully resumes that pair through
38735 with `ZELDA3_LIVE_RNG_DIAGNOSTIC_RESUME=1` (diagnostic only).
The repeated native run `target/native-overworld-prep-native` confirms the
same38732 frontier and dumps WRAM at38730..38732. It did not produce a
native paired checkpoint; do not substitute the receipt-driven source pair
when measuring native SPC state.
The source traces are diagnostic runs with A/V comparisons disabled, not
acceptance gates. Keep the candidate isolated while batching further fixes.

The byte-packing batch now advances native exact A/V through **38738**;
the first mismatch is **38739, video-only**, with audio still exact.
`target/native-byte-packing-scroll-native` and
`/tmp/native-byte-packing-scroll-native.log` record the 100001-frame attempt
(46.71s). Candidate binary SHA256:
`f8d062f7a7eff3769e9a2c33278fafc2c5bc563619c3b9bca328b2a9990075b1`.
The runtime remains uncommitted in `target/native-song-upload-worktree`.
This does not replace main's full 1581079-frame receipt proof.

New `ExtendedOamPackingProgress` records committed bytes and CPU cost within
one four-byte pass. Prefix/resume preserve already committed bytes, execute
the stateful suffix once, and sum to the atomic cost. The focused regression
pins the source's Y4/PC8620 boundary at7232 CPU clocks (412 within the pass).
The native overworld predictor executes an isolated pre-NMI source shadow
with actual host input seeded into the auto-joypad register. Zero input
incorrectly missed this workload. It predicts entry V219/C488 and the same
one committed byte at V225/C14, PC861f (398 in the pass); source entry is
V219/C478 and acceptance PC8620/V225/C18. Those small clock differences are
still unresolved; no compensating constant was added.

`target/native-byte-packing-source-display` resumes the paired source from
38001 through38735 and captures WRAM/VRAM. Comparison with
`target/native-byte-packing-display-trace` proves native OAM bytes $0800-$0a1f,
frame counter and latch match source at38730..38734 after the packing change.
The remaining display fix captures the next field for the native overworld
return, avoids the dungeon-specific retained OBJ cache, and uses the existing
`retain_completed_nmi_scroll_for_current_scanout`: Held NMI still executes
WritePpuRegisters. Before that last change, native host38733 retained scroll
[(1209,1161),(1138,1298)] instead of source [(1208,1161),(1137,1299)].
Composed captures are `target/native-byte-packing-presented` and
`target/native-byte-packing-receipt-presented`; their raw OAM, VRAM and CGRAM
agree at that host. OBJ latch storage/semantic cache representations differ;
do not assume that alone is a rendered mismatch.

The shared packing refactor retains **40000 exact receipt A/V frames** in
`target/native-byte-packing-receipt` (47.61s), on binary
`c227db7909f2389ba848235e9659e4e70ce26f73ab6a65ceddfa1ba69ebbc589`.
The subsequent change only extends the explicitly native completed-scroll
guard. Final-head engine suite:1772 passed,3 ignored in24.17s,
`/tmp/native-byte-scroll-lib-tests.log`.

Next: the final pointer-publication tail of NMI_PrepareSprites, $874e-$8780.
The same predictor reports PC876e at engine host38739 and PC8761 at38743,
after packing has completed; these currently return no continuation.
Fresh source `target/native-preparation-tail-source` resumes38001 through38745.
Decoded `/tmp/native-preparation-tail-source.jsonl` has run numbers relative
to the checkpoint: add38001 for comparison frames. Source run737 (38738)
enters preparation V216/C202, accepts Held NMI at PC876e/V225/C22 and returns
inside the handler atPC80c9/V225/C84; run738 reaches caller805d V227/C690.
Run742 accepts atPC8761/V225/C36; run743 reaches caller805d V227/C846.
The tail publishes head/body/travel-bird source-word pairs, then SEP/RTS;
its 610 CPU clocks are currently atomic in misc.rs. Split those committed
words and the remaining cost without repeating either animation countdown.

- `510da835`: grayscale caller finishes its held NMI before authoring the next
  palette; retires at main wait. Exposed earlier native frontier 14076.
- `2368fe51`: ground-item decoder return preserves the following Open NMI;
  native 14076 → 20202.
- `ed8f7863`: Big Key entry after a leading NMI attaches entry scroll to the
  current display capture; native 20202 → 23984 (audio).
- `bc212dec`: removed the room-specific live-SFX override in the SPC renderer;
  stair sounds use the NMI-sampled queue. Native 23984 → 24943 (video).
- `96e5f3e1`: supertile Sprite_Main return consumes its held NMI before the
  caller/suffix and prepares the next quadrant CPU slice at main wait.
  Native 24943 → 25054 (audio). This is scoped to the supertile chain;
  spiral callers retain their independent dispatcher-reentry schedule.
- `7b5db084`: guard head/body/weapon and follower drawing cycle annotations, plus their
  caller prefixes: native 25054 → 25868. ROM reference tests cover poses,
  clipping, follower movement/menu states and visibility. Route host 25031
  charges match the ROM profile exactly for all three guard draw routines,
  Follower_Main (7328), follower coordinate calls (1072), and the bank-5
  inactive wrapper (558). Short receipt comparison passes 25,900 frames.
- `4b7a9b16`: straight-stair fadeout uses the continuous measured Module7
  caller phase instead of the native room/countdown pause list. The source
  interruption is in NMI_PrepareSprites after Sprite_Main and LinkOam return.
  Native 25868 → 25922; 13 focused stair tests and 25,950 receipt frames pass.
  Across 25865–25869, native/receipt WRAM differ only at scratch $1f00.
- `b1839b4b`: straight-stair BG34 conversion and sprite-reset returns now consume their
  Held NMIs before the resumed callers. The reset uses a measured partial
  garnish-clear checkpoint. Native 25922 → 25925; 15 focused stair tests and
  26,000 receipt frames pass. WRAM25920–25923 agree apart from scratch.
- `51761a5a`: straight-stair quadrant callers retire at main wait, so the next Open NMI
  publishes pending uploads before the following palette iteration. Native
  25925 → 26516; all16 focused stair tests and26,530 receipt frames pass.
  WRAM25924–25928 agree apart from scratch.
- `33ff03af`: movable-mantle drawing, its bank/inactive caller, and shared OAM correction
  costs now follow the ROM instructions. The reference matrix checks exact
  cycles and OAM bytes for clipping, tile counts and size flags, including the
  complete dialogue-time mantle caller. Guard/follower references also pass.
  Native26516 →27888; receipt27,920 frames pass. WRAM26510–26517 now agree
  apart from scratch. Evidence: `target/mantle-cycle-native` and
  `target/mantle-cycle-receipt`.

- `fc3abbe9`: dungeon-map terminal fade measures the direct INIDISP write from the leading
  NMI through Sprite_Main. The output row accounts for the Snes9x render event
  at master cycle512. Native27888 →27926, including the earlier14286 map
  entry. Both focused regressions and27,950 receipt frames pass. Evidence:
  `target/map-blank-native2`, `target/map-blank-receipt`. The CPU plan still
  requires the development ROM; it is not a completed ROM-less timing model.

- `4fe5b026`: dungeon-map room drawing measures its complete caller through main wait
  instead of always adding the one-NMI pause from the first map visit. Native
  27926 →28836; all12 map tests and28,900 receipt frames pass. The first candidate stopped at the
  drawer RTL and missed the caller/sprite-preparation interruption at14321;
  the accepted candidate includes that suffix. Native evidence:
  `target/map-room-native2`, `target/map-room-receipt`; source: `target/map-room-source`.

- `504a9f4a`: animated BG scanout follows the main-entry phase: changing the dispatcher
  cannot undo a completed leading-NMI upload. The old cross-phase selector
  restored stale tiles on the28836 gameplay-to-spiral transition. All23
  animated regressions and29,580 receipt frames pass. The100k native probe
  reaches an existing fail-closed reset checkpoint at host29550 ($09:c255).
  Source29550 confirms `Disable(SpriteLimitInstanceCleared)`; use that existing
  progress token. Do not add an atomic reset or a frame exception. Bounded
  native29,540 passes: `target/animated-entry-native-prefix`; receipt:
  `target/animated-entry-receipt`; source: `target/native-29550-source`.

- `163f1fa6`: straight-stair reset measurement recognizes the existing Disable tokens at
  $09:c252 and$c255. No new runtime capability: the source29550 confirms
  SpriteLimitInstanceCleared. The expanded prefix regression preserves seeded
  counters/garnish until resume; native29547–29552 WRAM matches apart from
  scratch. Native reaches31363; receipt31,400 passes. Evidence:
  `target/reset-disable-native`, `target/reset-disable-receipt`.

- `caf91b9e`: quadrant caller batch: cached Sprite_Main, post-Sprite_Main, filtered build
  and upload returns consume the held handler before completing their callers,
  then leave queued uploads for the next Open NMI. A CPU interruption before
  NMI_PrepareSprites retains and executes the whole pending common suffix.
  Native31363 →31367; all14 quadrant regressions and31,400 receipt frames pass.
  WRAM31357–31367 matches apart from scratch $1f00. Evidence:
  `target/quadrant-batch-native`, `target/quadrant-batch-receipt` and
  `/tmp/quadrant-batch-final-tests.log`. Source: `target/native-31363-source`.
  The initial animated-BG hypothesis at31367 was disproved: its raw tiles
  already agree. Source scanlines present BG1 scroll65460 while native retained
  65458. Interrupt_NMI writes scroll outside its latch-gated DMA body.

- Landing/return batch: retain completed held-NMI scroll on the current field;
  classify $0085fc as NMI_PrepareSprites entry; measure native landing states
  from the actual pre-NMI state instead of a calibrated entry-time interval;
  retain only the dedicated preparation continuation after LinkOam/HUD return;
  attach the interrupted OBJ cache to its current return field, leaving the
  next Open NMI free to publish new Link art. Native31367 →33895. Source:
  `target/native-31363-source`, `target/native-33322-source`,
  `target/native-27215-source`; CPU trace `/tmp/native-33322-cpu.log`.
  All1762 library tests pass (3 ignored): `/tmp/landing-batch-lib-tests.log`.
  Native evidence: `target/prep-cache-owner-native`. WRAM27210–27217 and
  33318–33325 match apart from scratch $1f00. Next source window:
  `target/native-33895-source`. The receipt batch check is
  `target/landing-batch-receipt` (33,920 exact frames); full-route acceptance
  remains deferred.
  Reusable composed-display dumps now include five-byte logical/preview CHR
  identities, documented in CLAUDE.md.

- Ordinary spiral second-palette caller: complete the carried Held NMI
  before the second walk and common suffix, then retire at main wait. The
  previous synthetic trailing Open NMI consumed queued uploads too early.
  At33895, native animated tiles differed from the source by791 pixels;
  the receipt tiles matched. Native33895 →36022; WRAM33888–33902 now
  matches except scratch $1f00. Both palette-return regression variants
  pass, as do all1762 library tests (3 ignored). Evidence:
  `target/spiral-held-native`, `target/spiral-held-receipt` (36,040 exact),
  `target/native-33895`, `target/native-33895-source`, and
  `/tmp/spiral-held-lib-tests.log`.
  Next audio window: `target/native-36022` versus
  `target/spiral-held-receipt`, source `target/native-36022-source`.
  Native is one glyph behind at36014 and clears SFX2 at36018 while the
  receipt retains12. Investigate the dialogue CPU budget and NMI boundary;
  this is diagnosis, not a proven cost correction.

- Dialogue drawing/equipment cost batch: price Zelda's banked caller and
  crystal-maiden drawing wrapper, deferred OAM allocation and its positional
  checks, and Link's equipment-VRAM and signed-X-offset helpers. The source
  trace places the last click at $0E:CAC9 across NMI at36018; native had
  already begun drawing that glyph. Missing caller work let it write the
  sound queue too early. The combined corrections move native36022 →37590.
  All112 Zelda drawing/clipping/allocation ROM cases and every equipment
  table entry/signed byte offset pass; all1764 library tests pass (3 ignored).
  Evidence: `target/equipment-batch-native`, `target/native-36022-cpu`,
  `target/native-36022-ledger`, `target/native-36022-profiles`, and
  `/tmp/equipment-batch-lib-tests.log`. Drawing-only and drawing/allocation
  probes retained36022; the combined batch is the advancing candidate.
  The receipt check `target/equipment-batch-receipt` passes37,620 exact
  frames; full-route acceptance remains deferred until native100k.
  Next source window: `target/native-37590-source`. The source is completing
  dungeon-exit spotlight work and interrupting LinkOam; diagnose publication
  and the interrupted caller before changing costs or receipt capabilities.
  `target/native-37590` has comparison WRAM37584–37598 and composed display
  hosts37589–37594, matching the receipt batch's diagnostic window. At37590,
  live WRAM agrees except scratch; displayed VRAM, BG VRAM, CGRAM and OAM
  agree, but the OBJ cache differs. Several caller-return hosts also leave
  native $12 latched while the receipt clears it. Compare cache publication
  against actual source OBJ tiles before treating the cache difference alone
  as proof of its owner.
  Follow-up source comparison rules out those OBJ-cache differences: all105
  visible source tiles (6,720 pixels) match both decoded caches. The captured
  reserved table differs at $170f2–$17127 while live WRAM agrees. The port
  hardware-facing dynamic table is at $1dba0, not $17000 or raw $1b00.
- Retained spotlight HDMA ownership: `RetainPublished` discarded the measured
  active-field window receipt while keeping OAM/VRAM. A whole-table fallback
  then exposed the following circle (row128) instead of the source field
  (row121). Transfer that independent measured receipt to the retained
  snapshot and retire its obsolete table fallback; keep the following receipt
  queued. Native37590 →37690, audio exact. The regression checks retained
  OAM/VRAM, current window, following window, and unchanged CPU RAM.
  Evidence: `target/spotlight-retained-native`,
  `target/spotlight-retained-receipt` (37,710 exact),
  `/tmp/spotlight-retained-lib-tests.log` (1,765 passed,3 ignored),
  `target/native-37590-owner`, `/tmp/native-37590-boundaries.log`.
  Presented-state diagnostics now include composed scanline windows and
  spotlight ownership. Next frontier: `target/native-37690-source` and
  `target/native-37690`; source is force-blank in pre-overworld overlays.
  The next mismatch is a caller-timing gap, not remaining spotlight pixels:
  native completes properties on enginehost37663 and enters overlays on37664,
  then finishes the screen build on37687. Source properties return at
  comparison37662, followed by NMI-masked continued-call hosts37663–37685;
  the common suffix and Open NMI arrive at37686, overlays start37687 and
  return37691. Native incorrectly unblanks while source is still loading.
  `complete_pre_overworld_load_properties_after_sprite_reset_with_presence`
  ends in `LoadOWMusicIfNeeded` ($02:854c → $00:8913 → $00:8888). The audio
  owner already performs a non-atomic `SongBankHostTransfer`, but the native
  `FinishPreOverworldProperties` arm prepares sprites, clears $12 and marks
  main-wait immediately; only its Live-owner branch keeps the common suffix
  pending. Investigate coupling that native caller to the existing upload
  completion and NMI mask, including the exact return-host phase. Do not add
  another calibrated 24-frame hold: the byte/ack protocol already owns time.
  The native properties stage and source marker agree after converting
  enginehost37663 to comparison37662. The native overlays duration is also
  still a fixed seven slices versus five source hosts here; price that caller
  independently after restoring the missing upload wait. Audio advances after
  gameplay for each host, so merely checking whether the preceding audio host
  finished uploading risks retiring the caller one host late. Preserve the
  completion timestamp within the field, not just a transfer-busy boolean.
  Source proof: `target/native-37690-return-cpu/upload-window.bin` and
  `/tmp/pre-overworld-upload-source.jsonl`. On comparison37662, $02:854c
  starts at V31/C638 and $00:8913 at V31/C900. $02:8552/$8555 clear
  $4200/$420c before the APUI0 $ff request. The upload returns to $00:8923
  on37686 at V251/C676; restoring $4200 accepts Held NMI at V251/C822,
  resumes at V253/C1128, reaches the $00:805d latch-clear boundary at
  V0/C906, then accepts Open NMI at V225/C12. Source $13 remains zero;
  that software byte is not the hardware NMI mask. The diagnostic trace
  contains complete evidence through37692; its final37693 return was cut
  off by the trace frame filter, so this is diagnostic evidence, not a pass.
  Future trace filters should end one host beyond the requested frame count.
  Never filter $00:8034 for this investigation: it is a hot busy-wait loop;
  the abandoned trace was stopped and its 4.6GB file removed.

**Measured pre-overworld overlays.** Native now measures the full
PreOverworld_LoadOverlays caller from the leading NMI through main wait,
including overlay-dependent map decoding and the common suffix. The old
fixed six-slice delay was two hosts too long for screen$13/progress2:
source runs37687–37691 cross four Held NMIs. The regression executes the
pinned ROM, checks four crossings, verifies live RAM is unchanged, and checks
the zero-crossing special-area branch retires its pending measurement.
Receipt ownership keeps its existing authority; measurement is native-only.
The corrected duration exposes the missing song-upload wait two hosts earlier:
native37690 →37688, video-only. This is the user-authorized source-fidelity
correction, not claimed native frontier improvement. Evidence:
`target/overlays-final-native`, `/tmp/overlays-final-test.log`,
`target/overlays-final-receipt` (37,710 exact), and
`/tmp/overlays-final-lib-tests.log` (1,766 passed,3 ignored).
The interrupted upload is not repaired by this commit. Next implementation
needs two independent facts: the main-CPU APUI0 command position (the current
SPC clock schedules unqualified main writes at the end of the audio window)
and the final upload acknowledgement/port-clear timestamp. Connecting only a
transfer-busy boolean to the caller risks returning one host late because
audio currently advances after gameplay. The existing protocol in
`spc_driver_clock.rs` already prices the handshake, block headers, bytes and
port clears; extend that owner rather than introducing a fixed upload delay.
For a native return model, measure the properties prefix through $02:855d,
then preserve the CPU suffix's actual interrupt phase after the receiver
returns. Source $00:88ff PLP + $00:8900 RTS takes70 master clocks after the
last port clear; CLI/RTL/LDA/STA in the overworld caller takes104 more to
restore $4200. Refresh stalls still apply; an active-field return need not
accept the immediate Held NMI seen in this particular trace.

The audio mismatch at 25054 came from missing drawing work before the text
renderer. The original enters VWF at v=50 on host 25048; the old ledger left
native about one glyph ahead by host 25050, moving the final click before
its held NMI. The drawing annotations correct this without an audio override.

**Repaired reset at25922.** The cached receipt says `SpritesDisabled`, but a fresh
CPU trace proves host25921 actually returns at $09:c28c with X=10, and25922
accepts its Held NMI at $09:c28d with X=9. Ten garnish slots remain to clear.
Do not copy the coarse receipt into the native schedule. The new native
candidate measures the written garnish slot and carries that partial clear.
The preceding BG34 conversion return also consumed its Held NMI too late;
the candidate now matches25920 WRAM apart from scratch. Source receipts:
`target/native-25922-source`; live CPU trace `target/reset-phase-source/window.jsonl`;
native probe `target/native-25922`, receipt probe `target/stair-phase-receipt`.
The working-batch proof is `target/reset-phase-native6` and
`target/reset-phase-receipt`; full-route acceptance remains deferred. The short
live trace ended at its configured trace cutoff with a missing final return;
its25920–25923 CPU evidence is complete, but it is not an acceptance run.
The quadrant upload chain (states5–8) is repaired. Evidence:
`target/straight-quadrant-native`, `target/straight-quadrant-receipt`.
The room51 dialogue audio mismatch is repaired. Source receipts:
`target/native-26516-source`; original WRAM/VWF/cycle-ledger probe:
`target/native-26516`, `target/native-26516-ledger`, `target/native-26516-profiles`.
**Repaired:27888**, dungeon-map forced-blank write.
**Repaired:27926**, dungeon-map drawing caller interruption count.
**Repaired:28836**, main-entry animated-BG ownership.
**Repaired:29550**, straight-stair reset Disable progress (above).
**Next:31363**, continued caller return; source `target/native-31363-source`,
probe `target/native-31363`, receipt `target/reset-disable-receipt`. Earlier source:
`target/native-28836-source`; native/receipt probes below.
Source: `target/native-27926-source`; native probe: `target/native-27926`;
receipt probe: `target/map-blank-receipt`. Source receipts: `target/native-27888-source`; native
probe `target/native-27888`; receipt probe `target/mantle-cycle-receipt`.
Prefix evidence: `target/vwf-25054-source`,
`target/drawing-batch-ledger`, `target/drawing-batch-profiles`; chronological
working notes: `target/native-100k-batch/progress.md`.

## Previous starting point: frame 23945

The completed batch fixes cached sprite-conversion retirement (`6704a050`),
dungeon NMI_PrepareSprites main-wait retirement (`63443ece`), and an obsolete
room-specific spiral audio sample (`e70152ae`). The native frontier advances
23203 → 23206 → 23935 → **23945**, now video-only. The 24,100-frame cold
live-Snes9x check, 200k/full receipt gates, WRAM goldens and endpoints all
pass. See "Completed quadrant-return and spiral-audio batch" in
`romless-exact-play.md` for source contracts and regressions.

At 23940-23944 native and receipt WRAM differ only at known scratch $1f00.
At 23945 both modes remain in 07/0e/0f. Native/receipt differences are
$12=01/00, $15=00/02, $16=00/01, $19=00/58 and scratch $1f00=00/01.
Original host 23945 completes its carried handler, Sprite_Main and common
suffix, then accepts an Open NMI. Investigate publication at that trailing
acceptance after the grayscale palette caller returns. This is a starting
diagnosis, not a proven cause; do not add another frame/room exception.

Use `ZELDA3_DEBUG_PRESENTED_FRAMES=23946` and
`ZELDA3_DEBUG_PRESENTED_DIR=<dir>` to capture fully composed display VRAM,
OBJ VRAM, CGRAM, OAM, presentation RAM and registers. These dumps describe
the renderer's selected generations; use WRAM dumps separately for live CPU
state. See CLAUDE.md for the reusable diagnostic added in `0a99f7f9`.

Evidence: `target/quadrant-batch-validation` (compact proofs and
`next-frontier.md`), `target/quadrant-batch-native-final`,
`target/quadrant-batch-cold-av`, and `target/quadrant-batch-next-source`.
Earlier quadrant/spiral probes remain under `target/quadrant-*`. Rebuild
`target/alt` from `main` before beginning the next batch. The earlier
Module0F entry envelope and pending within-row CPU decrement remain
separate source-backed work; do not retune them to move this frontier.

## How to diagnose a frontier frame

1. **Rust against Rust first.** Run the same binary with and without
   `ZELDA3_CACHED_AV_NATIVE_TIMING=1`, dumping `ZELDA3_DEBUG_WRAM_FRAMES`
   at the frontier and a few frames before, and compare the dumps. The
   receipt path is the ground truth. At the repaired 8889/8890 boundary only
   the known scratch byte remains. At repaired frame 4785, matching CPU tables
   narrowed the mismatch to display publication. At current frame 23945,
   trace the trailing Open acceptance and compare its composed palette and
   display generations; live module state agrees but four control bytes
   differ. `$1f00` differs benignly; interpret `$12` against the actual NMI
   boundary rather than dismissing it as scratch.
2. **Then the subsystem's own trace.** For dialogue:
   `ZELDA3_DEBUG_SCROLL_STAGE=1 ZELDA3_DEBUG_SCROLL_RETAIN=1` in both
   modes gives the scroll phase machine's decisions side by side;
   `ZELDA3_DEBUG_VWF_BUDGET_FRAME=<host>` gives one frame of per-glyph
   receipts; `ZELDA3_DEBUG_VWF_BUDGET=1` gives all frames, short windows
   only.
3. **Then the original hardware.** Drive the instrumented Snes9x core
   directly; the recipe and its gotchas are in
   `docs/parity/romless-exact-play.md` under "Ground truth for the
   fresh-entry prefix". At most ten PC filters, the frame filter must
   start at 0, and `./parity microscope --cold` refuses this route, so
   call the comparison harness yourself with the `target/alt` binary.

Engine host N corresponds to Snes9x run N−1.

## Batch fixes before full-route validation

The user requested batching on 2026-09-11 because a full-route check takes
about 25 minutes. The user strengthened this on 2026-09-11 to target native
exact A/V of at least100,000 frames. On 2026-09-12 the user allowed full
parity checks when needed, including before100k. Use focused regressions and short
receipt checks during development, with one root cause per commit. Run the expensive acceptance
and promotion sequence once for the completed batch, rather than once per
fix. End a batch sooner if an acceptance regression cannot be isolated
confidently. An explained backward move of the native frontier is not itself
a reason to reject a source-proven correction or end the batch.

For each fix, add reference-backed regression coverage, run the relevant
tests, and measure the native frontier again. Run a focused receipt-driven
cached A/V comparison through the affected window as well; a native frontier
improvement alone does not prove acceptance was preserved. Keep each fix
independently revertible, and do not commit unresolved experiments. When
committing these intermediate fixes, `ZELDA3_PRECOMMIT_SKIP_SNES9X=1` avoids
the legacy long gate; the hook's build and standalone smoke still run, and
the complete acceptance sequence below remains mandatory before merging.

Keep `main` at the last fully validated batch while work continues on the
working branch. Freeze the completed batch's binary and commit for its full
run; do not rebuild that binary during comparison. Preserve serial GPU runs
and use `target/alt` for independent development builds.

## Validating and promoting a completed batch

1. `cargo check -p zelda3`, then `cargo test -p zelda3 --lib --no-run`
   with zero warnings. That build is the dead-code detector.
2. `cargo test --profile parity -p zelda3 --lib`.
3. A 200,000-frame cached comparison with the WRAM goldens and the
   endpoint compare.
4. The full route, then
   `./parity promote --cached-av <run> --binary target/parity/zelda3`. The
   tree must be clean at the validated commit; if the branch has moved,
   promote from a detached worktree at that commit and copy
   `routes/full_run/parity-frontier.json` and the receipt manifest back.
5. Commit the evidence, move `main`, prune the run directories.

## The backlog after the current native frontier

- March the frontier. Each divergence is now a single named mechanism.
- Finish the ledger census. The remaining classes are hosts where the
  profile records two engine iterations against one ledger host, the split
  between `Sprite_ExecuteSingle` and the inactive-sprite path, room-object
  drawers, and the interrupt handler's own attribution.
- Replace the eleven ROM-driven timing plans with native models. The
  dialogue initialization plan is closest: its cost is a fixed graphics
  decompression plus a message-dependent part, and the decompression,
  character-buffer and variable-width-font models are already exact. The
  room-load plan is hardest and needs a per-object cycle model.
- Only then does live play with no ROM become reachable; it currently
  starts from a different boot path with no receipts at all.
