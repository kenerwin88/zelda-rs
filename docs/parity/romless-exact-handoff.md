# Romless exact play: handoff

Read this first, then `docs/parity/romless-exact-play.md` for the program's
history and evidence, and `docs/parity/cycle-ledger-recipe.md` before
annotating any routine.

## Current native frontier — 54,043 (audio)

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

### Next: wire the measured upload command; retain the unresolved phase evidence

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
