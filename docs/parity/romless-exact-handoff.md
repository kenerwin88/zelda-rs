# Romless exact play: handoff

Read this first, then `docs/parity/romless-exact-play.md` for the program's
history and evidence, and `docs/parity/cycle-ledger-recipe.md` before
annotating any routine.

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

## Last accepted main baseline

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
Work remains on `fix/romless-spiral-palette-return`; `main` remains the accepted
baseline above. No push. Short receipt comparisons protect the shared path
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
