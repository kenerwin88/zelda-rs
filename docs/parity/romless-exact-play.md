# Toward exact ROM-less play

> Handing this to someone new? Read
> [`romless-exact-handoff.md`](romless-exact-handoff.md) first: it carries
> the current state, the rules, the next task and the diagnosis recipe.
> This document is the program's history and its evidence.

## Direction

The no-argument launch (`zelda3` with no ROM) must reach the same
frame-exact behavior as the parity route without any ROM bytes at
runtime: the timing the engine still takes from the original code has
to become native Rust logic that matches it exactly. Embedding the ROM
in the binary was tried and rejected; this document records what the
receipt-less path looks like today, the fixes this batch made to it,
and the program that closes the gap.

## Where exactness comes from today

Full-route parity is proven for the engine driven by per-host timing
receipts from the pinned Snes9x trace core (`OriginalTimingHostReceipts`,
installed by the cached comparison from the cache's
`original-timing-host-receipts.jsonl.zst`; owner state `Live`). Live play
installs no receipts: the owner stays `PendingColdStart` and every host
runs the translated "atomic" schedule, one main iteration and one NMI
per frame. With a ROM loaded, eleven `RomCpuTimingRun` plans execute the
original code on the in-repo 65816 to decide a handful of boundaries
(dialogue initialization, dungeon module 7 advances, cached sprites,
spotlight tables, the poly-thread intro); without a ROM those plans are
unavailable and the flag-off path runs its unmeasured schedules.

In the first 200,000 route hosts, 20,657 (10.3%) carry timing the atomic
schedule cannot reproduce: a held NMI (13,522 hosts), a main-loop
interruption (1,937) or a continued call stack (16,732). Those are the
hosts a native timing owner has to predict.

## The program

1. **Replace the eleven ROM-CPU plans with native cycle models.** Each
   plan measures one routine's cycle cost from a known entry raster to
   the next NMI. The route cache holds the ground truth for every host
   (`ZELDA3_DEBUG_INSTALL_RECEIPTS` prints the vector), so each native
   model can be table-tested against thousands of measured instances
   before the ROM path is deleted.
2. **A native timing owner.** `OriginalTimingOwnerState::Live` was
   designed so that "a native timing owner can later publish the same
   Zelda-level receipt vocabulary without changing gameplay". The owner
   needs a per-host cycle budget model of the translated main loop (the
   cost of each module iteration as a function of its state) to decide
   where vblank lands: NmiAccepted gate, MainLoopInterrupted boundary,
   CallStackContinued. The 1.58M cached receipts are the regression
   oracle: `./parity receipt-compare` already checks sampled semantic
   receipts, and a native owner can be run against the whole cache
   without Snes9x.
3. **Validate the receipt-less path continuously.** The standalone smoke
   now walks the route's own inputs through live play
   (`ZELDA3_SMOKE_INPUT_LEDGER`, `ZELDA3_SMOKE_SRAM`, `ZELDA3_SMOKE_TRACE`,
   `ZELDA3_SMOKE_ROM=<rom>` to compare against a ROM-loaded launch). It
   presents each frame through the play loop's headless publication
   pipeline and hashes WRAM every 100 frames, so two launches can be
   compared frame for frame and a crash names its host.

## Receipt-less fixes in this batch

Walking the whole route's inputs through the receipt-less ROM-timing
path (the `zelda3 <rom>` live path) was the first long run of that path.
It found:

- **The message-line scroll lag frame.** `lane_rom_startup_run_main`
  claimed a returned main iteration while a scroll copy was still in
  flight and ran an ownerless leading NMI into the copy, which the NMI
  lifecycle assertion rejects (the crash the interim ROM-less fix
  avoided by turning timing off). The two leading-NMI lanes now require
  the scroll CPU to be idle, so the lag frame runs the ordinary
  game-loop path that copies the remaining passes around the frame's
  NMI.
- **A staged scroll completion at an ownerless leading NMI.** After the
  return-only slice stages the completed text, the next host's leading
  NMI publishes it through its BG3 DMA (as the open NMI does under
  receipts). The ownerless-NMI assertion now admits the two staged
  phases; its second check still proves the publication token was
  claimed. Under receipts these hosts never reach that lane, so the
  receipt-driven route is unchanged.
- **Dungeon-exit spotlight entry envelope.** Without a timing authority
  `Module0F_SpotlightClose` plans the iris close from a guessed raster
  entry envelope and asserted that both ends of the envelope give the
  same plan. At walk host 173,4xx they differ in whether Link's position
  integrates before the first NMI, which aborted play. The plan now keeps
  the earliest entry's plan, carries the widened next envelope, and
  reports the disagreement under `ZELDA3_DEBUG_SPOTLIGHT_ENVELOPE`. A
  timing authority never consults the envelope.
- **OAM model after the Module09 transition staging.** The transition
  staging runs Sprite_Main, LinkOam and the HUD refill provisionally,
  restores RAM and the native state, then keeps only the staged OAM
  bytes in RAM, which left the native OAM model one generation behind
  RAM (a debug coherence check fired at walk host 117,9xx). The two
  restore points now re-adopt the staged shadow and packed extended
  table into the model. The bridge writes through per byte and the whole
  model is never bulk-projected on the frame path, so no RAM byte
  changes.
- **Memorized-tile coherence check.** The smash and 32x32 map-update
  writers store an entry before raising the count, so the native model
  can hold one slot more than the count implies; the debug coherence
  check compared against a count-sized reload and fired past 32
  memorized tiles on one screen. The check now reloads the model's own
  extent (`load_from_ram_sized`). Debug builds only; no projection
  changed.

## Library suite time

One test, `tile_attributes_match_frozen_runtime_effects`, spent 104 of
the suite's 113 CPU seconds (131,072 tile executions, each digesting two
WRAM images); the other 1,705 tests under 50 ms sum to 1.1 s. Its
per-tile digests are independent, so it now splits the tiles across
threads while producing the identical frozen case text. The parity
profile suite runs in 13.7 s wall instead of 99.7 s; nothing was
removed.

## Evidence

Static checks are unchanged: the ownership scanner reports 0 same-mode
overlaps, 19 cross-mode overlaps and 60 overlapping bytes; RAM
readability (2,654 constants) and projection discovery pass; the dev
library test build has no warnings (no dead code).

Receipt-less walks of the route's inputs from its initial save: the
ROM-free launch walks 30,000 frames; the ROM-loaded live path walks
300,000 frames on the dev binary with every debug coherence check
enabled and the full 1,581,079-frame input ledger on the parity binary
is recorded below.

Parity binary
`ce3b59a2420c900868bb4968f96746143ab6dbee5f697a63a334739ecb532438`: the
200,000-frame cached comparison matched every video and audio hash, the
frame 60000 and 150470 WRAM goldens match, the 200,000-frame WRAM
endpoint is the recorded `dd45975c…` image, and all 1,721 library tests
pass under the parity profile in 14.66 seconds. The binary crate's
tests show the same fifteen environmental failures as before (missing
replay-bisect checkpoints).

The full route on the same parity binary matched every one of the
1,581,079 cached video and audio hashes in 1,551.38 seconds, all four WRAM
goldens match, and the route endpoint is the recorded `31619379…` WRAM
image; the run is promoted in `routes/full_run/parity-frontier.json` with
its receipt. The ROM-timing live path (a ROM loaded, no receipts) walked
the entire 1,581,079-frame input ledger on that binary without a panic.

## Rule discovery over the route (LogicPearl experiment)

`ZELDA3_DEBUG_HOST_FEATURES=<csv>` makes the cached comparison write one
row per host: twenty engine-state features read at the host boundary
(module triple, indoors, Link handler state, room and screen, frame
counter parity, the NMI latch `$12`, NMI subroutine `$17`, INIDISP copy,
the BG-from-VRAM and CGRAM-update flags, sprite state counts, active
ancillae, the VRAM upload cursor, and the previous host's class) and the
timing class the oracle receipt carried: `interrupted` (a main-loop
interruption), `continued` (a continued call stack), `held` (an NMI
accepted with the latch held) or `open`. The first 86,762 route hosts
(a dev-binary run; see the finding below) hold 76,520 open, 8,023
continued, 1,325 held and 894 interrupted hosts.

LogicPearl 0.1.5 (`logicpearl build --action-column host_timing
--default-action open`) learned five rules in 26 minutes (2 GB, one z3
selection) with 97.5% training parity. Scored per class against the
receipts:

| class | rule found | recall | precision |
|---|---|---|---|
| continued | `$12` (NMI latch) set at host entry | 1.000 | 0.974 |
| held | CGRAM-update flag, or VRAM cursor above 0xccad | 0.238 | 0.543 |
| interrupted | module 0x10 with NMI subroutine 4 | 0.016 | 1.000 |

The continued rule is the mechanism itself: a latch still set at the
host boundary means the previous iteration has not returned, so the
native owner can read that class from RAM today (the 218 misclassified
hosts are interruptions that also leave the latch set). Held and
interrupted hosts are the cycle-budget cases: nothing in the boundary
state says how much work the frame will do, so no rule over these
features separates them. That is the expected shape: discovery names
the state-marker classes immediately and confirms which classes need a
cost model, but it cannot invent the arithmetic. Its value for the
program is the mismatch listing it produces for a candidate cost model,
not a learned policy.

Finding from the dump run: on a dev build (debug assertions on) the
receipt-driven comparison stops at host 86,762 in
`rtl_dungeon.rs:394`, a scheduler-shape `debug_assert!` that the
cached-sprite room-load continuation is scheduled at the boundary just
armed. The parity binary carries no debug assertions and the promoted
route is exact there; the disagreement between that check and the
receipts is open for a later batch.

## ROM-CPU plan census (the measurement the native models replace)

`ZELDA3_DEBUG_ROM_CPU_PROFILE=<dir>` makes every `RomCpuTimingRun` write a
JSON profile when it is dropped: total master cycles, DMA cycles,
instruction count, NMI entries, inclusive cycles per subroutine (JSR/JSL
and interrupt entries tracked against RTS/RTL/RTI) and exclusive cycles
and execution counts per instruction address.
`scripts/rom_cpu_profile_summary.py <dir>` groups the profiles by plan
and names the subroutines through the ROM symbol table. Over the first
200,000 route hosts (2,699 shadow runs, the comparison itself still exact):

| plan entry | runs / hosts | total master cycles | what dominates |
|---|---|---|---|
| `$00:8034` main-wait schedule (dungeon submodule, Module09) | 1,922 / 1,284 | 54k .. 6.78M, 1,202 distinct | supertile transitions, palette filters, iris tables, sprite GFX set loads, Sprite_Main |
| `$00:8051` dungeon palette caller | 451 / 451 | 60k .. 298k, 136 distinct | ApplyPaletteFilter/FilterColors (60%) |
| `$00:f800` Module0E dialogue initialization | 218 / 109 | 1,983,900 .. 1,984,204, 35 distinct | Attract_DecompressStoryGFX (78%, a fixed decompression), then the message-dependent Text_LoadCharacterBuffer and RenderText commands (70k .. 280k) |
| `$02:8a26` supertile transition room load | 106 / 53 | 4.50M .. 5.90M, 55 distinct | LoadTransAuxGFX_sprite decompression (52%), Dungeon_LoadRoom object drawing (42%) |
| `$02:8586` Module1B spawn select | 2 / 1 | 1.694M | the same fixed decompression |

The dialogue plan is the first native model: its cost is a fixed
decompression the ROM performs from fixed data, plus a message-dependent
part small enough that the schedule key `(prefix crossings, caller
crossings, following-NMI operands)` stayed `(4, 5, true)` on every route
instance. Data-dependent constants (decompression of fixed graphics,
per-message character-buffer costs) can be computed once from the ROM at
asset extraction and shipped in the asset pack like the assets
themselves; only control-flow costs need native cycle models. The room
load plan is the hardest: its cost is the room's object list drawn
through RoomDraw, and needs a per-object cycle model.

## First native cycle model: the graphics decompressor

`crates/zelda3/src/cycle_models/decompress.rs` prices the ROM's
`Decompress` routine (`$00:E79E`, entered through `Decomp_spr` or
`Decomp_bg`) instruction by instruction from its disassembly: slow-ROM
fetches at 8 master cycles per byte, WRAM accesses at 8, internal cycles
at 6, branches at 22 taken and 16 not taken, and every command path
(direct copy, byte fill, word fill, increasing fill, back reference,
extended headers, the source bank wrap inside `GetNextByte`). Output and
back-reference accesses are priced by address with the pinned core's bus
rule, because a stream that runs past `$7F:FFFF` wraps into bank `$80`
where the register window costs 6. The model takes only what the routine
consumes: the compressed bytes, the sheet index (for the page-crossing
table loads) and the destination address.

Its test builds a shadow-CPU checkpoint at each entry point and runs
every sheet the cartridge's pointer tables name (108 sprite, 114
background: 222 streams, from 45 bytes to 1,705 bytes, including the
twelve raw sprite sheets the game never decodes) and demands equality:
222 of 222 match. The test skips when no ROM is present, so it is the
kind of check that runs in the validation chain, not in a ROM-free
build. This is the shape every plan component takes: a pure function of
the routine's inputs, verified exhaustively against the shadow CPU where
the cartridge enumerates the inputs, and against the route's profiles
where it does not.

In the dialogue plan the decompressor is 77% of the cost (the story
font: two sheets). Remaining components for a native dialogue schedule:
`Text_LoadCharacterBuffer` with `Text_DictionarySequence` (per message,
priced over the original dictionary-compressed bytes),
`RenderText_Draw_EmptyBuffer`, the `Text_Initialize` prefix, the NMI
handler bodies that interrupt the call, and the module's Sprite_Main,
LinkOam and HUD suffix before the return, which is the general
main-loop cost model the timing owner needs.

## Second native cycle model: the dialogue character-buffer loader

`crates/zelda3/src/cycle_models/text_buffer.rs` prices
`Text_LoadCharacterBuffer` (`$0E:C4E2`): the prologue that resolves the
message pointer, the per-byte loop (plain characters, the `$7F`
terminator, dictionary references expanded by `Text_DictionarySequence`
with the ROM's modulo-128 index wrap and do-while copy, and the commands
`$67-$7E` dispatched through `JumpTableLocal` to their seven handlers:
copy, copy with parameter, player name, window type, number digit,
position, color). The player-name handler is priced over the six packed
save-file words, including the glyph-mapping branches and the
trailing-space trim. Inputs: the message's original ROM bytes, the 128
dictionary word lengths, and the name.

Its test builds a shadow checkpoint at the routine, restores the
original message pointer table, and runs all 398 cartridge messages
(plus three names on the 59 messages that print the name): 516 of 516
match. With the decompressor this covers about 80% of the dialogue
plan's cycles from routine models; the message bytes and dictionary
lengths are what the asset extraction reads from the ROM, so a ROM-free
build can carry per-message costs or the original bytes in the asset
pack.

## The second gate: the native frontier

Acceptance stays the cached route's per-frame video and audio hashes.
The development gate is how far the engine's *own* timing reproduces
them: `ZELDA3_CACHED_AV_NATIVE_TIMING=1 ./parity cached-av <cache>` runs
the route's inputs without installing the cache's timing receipts (the
path live play takes) and stops at the first video or audio hash that
differs. That frame is the native frontier. With the ROM loaded it is
frame 8889 today (audio exact until then); WRAM dumps every 250 frames
show the receipt-less engine's state identical to the receipt-driven
engine's except the NMI latch and flags, the poly-thread scratch byte
`$1F00`, and briefly the BG animation countdown during the intro. The
first visible difference is a mechanism, not a state drift: at hosts
8885-8888 the dialogue renderer draws one glyph group per host fewer
than the ROM (its per-glyph costs are calibrated estimates), so the
line completes and publishes one host late.

There are two frontiers. With the ROM loaded and no receipts (the
`zelda3 <rom>` live path) it is frame 8889. With ROM startup timing off
(`ZELDA3_CACHED_AV_ROM_TIMING_OFF=1`, the no-argument launch's engine)
it is frame 1: the timing-off path boots differently from the first
frame, so ROM-free exactness is reached by making the timing-on path
stop needing ROM code (a native replacement for each of the eleven
plans, built from the cycle models and the ledger), not by improving the
timing-off path.

Frontier history for the ROM-loaded native path: 8889 (video, the
glyph estimates) → 4660 (audio: exact glyph costs against a whole-frame
resumed budget) → 2507 (video: entry budgets derived from a prefix the
ledger does not fully charge yet, which flipped the first scroll's
completion timing) → 8716 (audio) with the hybrid: traced entry
constants for a line's first host, the raster-derived budget (held NMI,
refresh and HDMA stalls) for resumed hosts. The engine's frame number is
the receipt host plus one. The remaining miss is one glyph too many on
the first resumed host after some line starts.

Each mechanism at the frontier gets a native rule, checked against the
cached receipts (hosts per operation, and read positions per host for
the dialogue renderer), and the frontier is measured again. The routines
a mechanism's frame count depends on get their exact costs from the
cycle ledger annotations, which is how the estimates are replaced.

## The cycle ledger program (status)

`crates/zelda3/src/cycle_ledger.rs` accumulates the master cycles the
original CPU spends in the code the translated engine runs: annotated
routines charge their assembly's block costs as they execute (see
`docs/parity/cycle-ledger-recipe.md`). `scripts/rom_block_costs.py`
prices a ROM routine's basic blocks statically with the shadow CPU's
rules (4,956 of 5,142 profiled instruction addresses exact, the rest
branch or page mixes), `scripts/rom_function_map.py` maps 2,651 of the
2,709 C-port routine addresses to their Rust translations, and
`scripts/cycle_ledger_check.py` compares each routine's ledger charge
with the shadow profiler's own-instruction cycles (inclusive minus
callee frames, frames tracked by stack depth) per host over a cached
comparison run with `ZELDA3_DEBUG_CYCLE_LEDGER` and
`ZELDA3_DEBUG_ROM_CPU_PROFILE` set.

Six batches of annotations were written in parallel worktrees (palette
filter, iris spotlight, sprite core, NMI/HUD/Link OAM, room draw,
graphics loads), verified, corrected and merged: 61 annotated routines.
Over the first 200,000 route hosts the check compares 37,039
host/routine pairs, 30,094 exact (81%), 843 skipped where the shadow
plan stopped inside the routine; every run stayed video- and
audio-exact, since annotations only add charges. An eighth batch added
the sprite handlers active on the route's dialogue hosts (uncle and
priest, green knife guard, mirror portal, the Sprite_DrawMultiple
family) and the Module0E prefix routines (lamp cone, rain, push block,
joypad read, stripes, incremental border), so that a fresh dialogue
line's entry budget can be derived from a fully charged prefix instead
of a traced constant. The profiler needed five rules to make the
comparison honest: frames live by stack depth (a jump-entered routine is
costed in the frame that jumped, and the ledger follows the same rule),
the interrupt entry sequence belongs to the handler frame, DMA bus time
is subtracted, a plan run spanning two main-loop iterations is summed
over two ledger hosts, and a raster wait spin is charged once with the
remaining passes subtracted. The remaining mismatch classes are named
per routine: sprite handlers reached by RTS dispatch (costed inside
Sprite_ExecuteSingle, unannotated), OAM allocation and coordinate
helpers reached through unscoped Rust paths, the NMI handler's joypad
wait and the $17 dispatch targets, and the room object drawers.

## Evidence for the annotation batch

Parity binary
`f8dd796af34f70a7e6b95df766525bd3de0575fe539380f359c31e6c5089b744`
(61 annotated routines, the cycle ledger and its tooling, the VWF cycle
model): the 200,000-frame cached comparison matched every video and
audio hash, the frame 60000 and 150470 goldens match, the 200,000-frame
WRAM endpoint is the recorded `dd45975c…` image, all 1,732 library tests
pass under the parity profile in 12.56 seconds, and the full route
matched every one of the 1,581,079 cached video and audio hashes with
all four goldens and the recorded `31619379…` endpoint (4,024.97 s: the
ledger's per-charge bookkeeping slows the comparison, to be tightened).
The run is promoted in `routes/full_run/parity-frontier.json`.

## Evidence for the VWF integration

Parity binary
`c4c073d1dc6450c6614ff8fd46a00bfec56c34ff098217637740e6e486044a32`
(exact VWF costs and raster-derived resumed budgets on the receipt-less
path, the receipt path bit-identical by gate, the ledger's plain-cell hot
path): the 200,000-frame cached comparison matched every video and audio
hash, the goldens and the `dd45975c…` endpoint match, all 1,734 library
tests pass under the parity profile in 13.11 seconds, and the full route
matched every one of the 1,581,079 cached video and audio hashes in
1,469.57 seconds (the ledger overhead is gone) with all four goldens and
the recorded `31619379…` endpoint. Promoted in
`routes/full_run/parity-frontier.json`.

## Ground truth for the fresh-entry prefix (instrumented core)

Once fresh VWF entries took the ledger-derived budget (the `ledger/vwf`
gate on `silent_calls`, merged as `26965378`) the native frontier fell
from 7330 to 2507: the derived budget on hosts 2337-2376 was 266,542
against a converted traced span of 224,608. The silent-call probe cannot
see the cause, because the missing work sits in routines that charge
something. The instrumented Snes9x core names it in one renderless run
(`ZELDA3_SNES9X_TRACE_EVENTS=frame,nmi,pc`, `ZELDA3_SNES9X_TRACE_FRAMES=
0-2340`, `ZELDA3_SNES9X_TRACE_PCS=<entry and return PCs>`, the
`--compare-snes9x-oracle` harness with the trace core, `--live-oracle-rng
--ignore-video --ignore-audio`; events carry `v` and `cycles`, so a
position is `v*1364+cycles` of a 357,368-cycle frame). Two rules for that
run: the trace core keeps at most 64 PC filters and the semantic adapter
appends about fifty of its own, so pass at most ten; and the frame filter
must start at 0 or the adapter misses its frame-0 receipts.

Route host 2337 (engine `frame_ctr_dbg` 2338, module 0E/02 in Link's
house, one active sprite), raster master cycles between PC events (about
82 per scanline crossed is refresh plus the live HDMA stall):

| span | ROM | ledger |
| --- | ---: | ---: |
| NMI accepted → handler return | 29,358 | 28,472 |
| handler return → `JSL Sprite_Main` | 6,292 | 7,152 |
| `Sprite_Main` entry → `Follower_Main` entry | 1,908 | ≈1,400 |
| `Follower_Main` → `Ancilla_Main` entry | 612 | 0 |
| `Ancilla_Main` → `Overlord_Main` entry | 10,418 | 0 |
| `Overlord_Main` → first `Sprite_ExecuteSingle` | 570 | 0 |
| 16 slots (15 inactive, slot 0 active) to `SpriteActive_Main` | 7,046 | ≈5,000 |
| `SpriteActive_Main` → `ExecuteCachedSprites` | 13,704 | ≈12,300 |
| `ExecuteCachedSprites` → return into Module0E | 2,428 | 2,166 |
| `LinkOam_Main` | 7,518 | 7,006 |
| return → `Hud_RefillLogic` entry | 234 | – |
| `Hud_RefillLogic` → `Hud_Update_IgnoreItemBox` entry | 990 | 1,166 |
| `Hud_Update_IgnoreItemBox` → return into Module0E | 10,334 | 3,624 |
| `JSL RunInterface` → `RenderText` → `Messaging_Text_Near` → VWF entry | 348 + 158 + 1,058 | exact |

The VWF handler enters at v=31 on every one of these hosts, 264,270
raster cycles before the next NMI acceptance; the `239,000` first-line
constant never described them. The missing ≈17k master cycles are
`Ancilla_Main` (no scope at all, ≈9.8k of work), `Hud_Update_IgnoreItemBox`
(≈6.1k short), and small residues in the Sprite_Main prologue and the
active slot's dispatch. Those went to the sprite and dialogue-sprite
annotation batches with the spans above as their targets; the derived
fresh-entry gate itself stands.

## Evidence for the charged prefix

Parity binary
`bf7f22f44e4281151a7e0a22b4c40700e82746ea45631eaf8e2a5481dc36adca`
(the `Ancilla_Main` subtree, the follower, garnish and overlord stubs, the
HUD heart, heart-draw and decimal helpers, and the `JumpTableLocal` frame
split at every annotated dispatch): the 200,000-frame cached comparison
matched every video and audio hash in 198.00 seconds, the goldens and the
`dd45975c…` endpoint match, all 1,735 library tests pass under the parity
profile in 13.35 seconds, and the full route matched every one of the
1,581,079 cached video and audio hashes in 1,579.44 seconds with all four
goldens and the recorded `31619379…` endpoint. Promoted in
`routes/full_run/parity-frontier.json` for `e3837c65`.

On the receipt-less path this state moves the native frontier from 2507 to
**8889**, past the previous best of 7330. Host 2338's charged prefix rose
from 70,892 to 86,206 master cycles since the NMI acceptance, against a ROM
span of about 87,500, and the ledger now reproduces the shadow profile
exactly, per call, for every routine in the `Module0E_Interface` prefix on
the Link's-house dialogue hosts.

The remaining divergence at 8889 is a resumed slice, not a budget size.
The ROM enters `RenderText_Draw_MessageCharacters` once on that host, one
scanline after the NMI, advances the message read position from 231 to 232
and renders no glyph at all; the engine renders the byte as a six-pixel
glyph. The line was already 153 pixels wide, so the candidate mechanisms
are the line-width test, a deferred glyph, and the click-and-wait path.

## The scroll's completion timing, and what 8890 still needs

The frontier at 8889 was not the glyph pipeline: the receipt path and the
native path render exactly the same glyphs, cursors and read positions
across 8885-8891. It was the message-line scroll. Whether
`RenderText_Draw_Scroll` returns before the next vblank was decided by a
constant threshold pinned to the traced later-line entry span, so it
compared a CPU-work headroom against a wall-clock constant. Once the
ledger charged the prefix properly the two scales diverged and the resumed
entry at host 8889 was scheduled `BeforeNextVblank` on 285,948 cycles of
headroom, staging the completed text a boundary early.

The decision is now the headroom against the call's own cost from the
scroll cycle model. One copy pass is 116,452 master cycles and only full
five-pass calls reach the decision, so none of them fits in a frame; the
comparison is kept rather than folded away because it is the physical
rule, and a cheaper call at a future call site would still be answered
correctly. The receipt path never reaches it: a live timing owner returns
earlier, on its source copy/return receipt.

What remains at 8890 is the shape of the continuation, not a budget. The
source finishes the copy passes and returns through RenderText and
Module0E inside one host, near V=196, then waits for the vblank whose NMI
opens the next host and publishes the staged text; the next scroll begins
after that publication. The translation instead spends a whole host on the
copies, a second return-only host, and stages one boundary later. Making
the continuation host return the call in place is not enough on its own:
the following host must then begin with a leading NMI, which the
receipt-less lane does not currently produce, and without it the next
Module0E iteration starts a scroll while the previous completion is still
staged. The fix belongs in the frame-lane scheduling of that post-return
vblank wait, not in the scroll machine.

## Evidence for the scroll completion-timing scale

Parity binary
`f9de4dacb5082c02884c3d4fb8200c52b24634b048f6337f4d6cc02869ea2c1c`:
the 200,000-frame cached comparison matched every video and audio hash in
193.22 seconds, the goldens and the `dd45975c…` endpoint match, all 1,735
library tests pass under the parity profile in 12.40 seconds, and the full
route matched every one of the 1,581,079 cached video and audio hashes in
1,497.36 seconds with all four goldens and the recorded `31619379…`
endpoint. Promoted in `routes/full_run/parity-frontier.json` for
`9b990d22`. The receipt-less native frontier is 8890.

## Native scroll return and the following main wait

This is a collapsed timed side-effect phase: `ZeldaState` finished the copy
without carrying its caller's return through `GameExecutionScheduler` to
the `$12` main-wait boundary. Copy completion, caller return and publication
therefore occupied the wrong host intervals.

The scroll now carries its remaining CPU work from entry to the continuation
host. That host first runs the held NMI, then compares the remaining copy
work, deferred handler exits and caller suffix with the raster-derived CPU
budget. A fitting return finishes RenderText, Module0E and the common suffix,
stages text behind the outgoing snapshot, and marks the existing scheduler's
`ReturnedToMainLoopBeforeNmi` phase. The next host consequently consumes a
leading NMI and publishes the staged text before another scroll can start.
The scroll publication machine and receipt-driven execution are unchanged.

The entry budget needed one additional correction: the caller still added
refresh/HDMA stall time back into the scroll headroom for the retired
wall-clock threshold. Those cycles cannot pay for copy instructions. Removing
that offset and charging the command dispatch before call entry preserves the
longer first-line case; otherwise the new return lane fails at frame 2508.

Cold instrumented Snes9x 1.63, pinned full-run cache inputs and initial SRAM:

| Source run | PC | V / master cycles | Meaning |
| --- | --- | --- | --- |
| 2506 | `$0E:CFE2` | 32 / 270 | first-line scroll entry |
| 2507 | NMI | 225 / 50, then 225 / 24 in the next field | two held acceptances |
| 2508 | `$0E:D0C2` | 228 / 460 | scroll returns after the second held boundary |
| 8888 | `$0E:CFE2` | 2 / 864 | scroll entry after resumed glyphs |
| 8889 | `$0E:D0C2` | 196 / 702 | copy returns |
| 8889 | `$00:F875` | 199 / 810 | Module0E returns |
| 8889 | `$00:805D` | 209 / 530 | common suffix reaches the latch clear |
| 8889 | NMI | 225 / 18 | open acceptance after main wait |
| 8890 | `$0E:CFE2` | 34 / 1214 | next scroll after publication |

The source copy receipts are `2+2+1` for runs 2506–2508 and `2+3` for
8888–8889. `native_scroll_return_preserves_the_original_post_return_nmi_wait`
tests both return decisions, the untouched frame counter, latch ownership,
leading-NMI carry, publication and safe adjacent scroll entry. The serialized
remaining-work field changes positional checkpoints, so the play checkpoint
magic is now `Z3RSPC22`.

To reproduce the timestamps:

```sh
scroll_cache=.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
ZELDA3_SNES9X_TRACE_EVENTS=frame,nmi,pc \
ZELDA3_SNES9X_TRACE_FRAMES=0-8900 \
ZELDA3_SNES9X_TRACE_PCS=0e:cfe2,0e:d0c2,00:f875,00:805d \
target/alt/parity/zelda3 --compare-snes9x-oracle \
  external/snes9x-libretro/local/snes9x_libretro_trace.dylib saves/zelda3.sfc 8900 \
  --input-script "$scroll_cache/input.txt" --load-sram "$scroll_cache/initial.srm" \
  --live-oracle-rng --compare-engine-state-from-frame 184000 \
  --ignore-video --ignore-audio --session-dir target/romless-scroll-source
```

With live oracle RNG, the harness overrides `ZELDA3_SNES9X_TRACE`: the binary trace is written
to the session's **`oracle-rom-random.jsonl`**, despite its filename. Decode
it with `scripts/snes9x_trace_format.py decode <path> --run-range 8888-8890`.
Use `--compare-engine-state-from-frame 184000` for this short renderless
probe, as reset WRAM differs before game initialization.

The native development comparison now first differs at **11444**, video
only. Receipt/native WRAM at 8889 and 8890 differs only at the known `$1F00`
scratch byte. The later frontier is Module0F spotlight close: at 11443,
`SPOTLIGHT_WINDOW_Y_BUFFER` (`$067A`) is `$000C` with receipts versus `$007E`
natively, and 44 HDMA-table bytes differ; at 11444 only the standing
`$12/$16/$1F00` differences remain while the presented image differs.
This identifies the next timing/publication investigation, not a proven
root cause. Partial pixel-copy WRAM within longer scrolls still uses the
existing coarse split; advancing A/V does not establish all native WRAM
or completely ROM-free execution as exact.

Validation for binary
`75aaed1128771423b6a47fbb51e468a1adca4dbfeb0346e904f4f5ddb4d9042b`:
the 200,000-frame cached A/V comparison is exact (200.45 s), both WRAM
goldens and the `dd45975c…` endpoint match, the cold live-Snes9x comparison
through 9,000 frames matches video and exact audio, and all 1,736 library
tests pass (two ignored). `cargo check` and the dev library-test build
have no warnings. The native frontier was reproduced at 11444 on this
same binary. Full-route promotion is recorded separately in the ledger.

The full-route check on clean commit `cbd3e99a` subsequently matched all
1,581,079 per-frame video/audio hashes in 1,509.50 seconds, all four WRAM
goldens, and endpoint
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The run and its receipt are promoted in `routes/full_run/parity-frontier.json`.
