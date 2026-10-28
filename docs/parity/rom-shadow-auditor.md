# Development ROM shadow cost auditor

Design at `1f93f138` (`codex/shadowplan`). The implementation and verified
M0/M1 stopping point at main `24b4fd3a` are recorded in [M1 result](#m1-result).
The decision is to keep shipped lagless timing in committed Rust prices derived
from ROM blocks. A separate development auditor may execute a locally supplied
ROM; it must never supply prices, holds or gameplay state to the game.

## What the auditor should establish

Run native logical replay and a private source CPU over the same logical
iterations. Emit executed instruction work by ROM PC, block, dynamic call and
native site, including negative differences. Reconcile **every iteration**, not
just missing holds. A difference is admitted as a pricing candidate only when
the two executions have a proven common entry, path and retirement boundary.
Otherwise emit an alignment or ownership residual with witnesses.

Use the corrected, read-only
`/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2` census:
1,473,123 iterations. The committed [Route pricing 2](cost-pacing.md#route-pricing-2)
receipt reports 2,919 missing ROM holds, zero excess and zero false positives.
This is a planning denominator, not a fresh census in this batch. The 2,913
missing *iterations* in its ranking are a different quantity. Do not mix the
ROM census with seeded live results or continuous live acceptance.

The [method](cost-pacing.md#runtime-rule),
[gap audit](cost-pacing.md#systematic-gap-audit-1) and
[Route ranking 1](cost-pacing.md#route-ranking-1) establish the constraints:
instruction work dominates the sampled gap; refresh/HDMA differences often
follow different instruction work; inclusive parents and aliases are not
additive prices. Route 1's 314-frame routing and 282-frame SingleLarge scores
are overlapping historical replacement scores, not available new prices.

## Existing pieces and reuse boundaries

Paths below are relative to the repository root; functions name the inspected
implementation, rather than a proposed interface.

| Existing code | Reuse | Limit to preserve |
| --- | --- | --- |
| `crates/zelda3/src/zelda_rtl/rtl_poly.rs::ZeldaState::begin_poly_shadow_frame`; `rtl_original_timing.rs::advance_poly_shadow_host` | Retained ROM interpreter across host slices; private thread scratch/bitmap; source entry and completion PCs | Specialized polygon model, with measured NMI swap/budgets and bounded stepping. It is not a whole-iteration lockstep auditor. Do not reuse its successful completion fallback on a step cap as audit success. |
| `crates/zelda3/src/rom_cpu_timing.rs::{RomCpuCheckpoint, RomCpuTimingRun::new, step, take_cpu_wram_writes}` | Detached WRAM/SRAM/PPU/DMA/APU-port copies, explicit program registers and stack bytes; bounded routine fixtures and write witnesses | `RomCpuCheckpoint` is a recipe, not a full machine checkpoint. `step` uses `cpu_run_opcode_timed`; callers own raster/DMA advancement. `disable_nmi_thread_switch` patches the private ROM: unsuitable for the auditor's unmodified-source lane. |
| `crates/snes/src/cpu_synchronous_executor/source_cpu.rs::{Snes9xColdCpuExecutor::from_lorom_reset_with_sram, step, run_until_main_loop_return_with_quiescent_step_hook, SourceCpuStepReceipt}` | Preferred retained, reset-driven CPU/APU owner with transaction/access timestamps and accepted interrupts; no external emulator process | Supported opcode/hardware scope must be checked. A host return is not a logical main iteration. An error or poisoned owner terminates the valid prefix. |
| Same file: `Snes9xCpuQuiescentCheckpoint`, `capture_quiescent_checkpoint`, `from_quiescent_checkpoint` | Exact resume at a validated instruction boundary, including timeline, source PPU reads, APU coroutine/clock and interrupt deadlines | Version 9 currently contains the ROM and duplicated APU RAM. Do not archive/ship these raw serializations; a future on-disk sidecar must bind external ROM identity and omit ROM bytes. |
| Same file: `rebase_private_timing_state_at`, `rebase_private_timing_regions_at` | Optional diagnostic transplant at a proved timestamp; retain source-private stack/scratch and all hardware owners | Equality of host number or WRAM is insufficient. The caller must prove registers describe the next ROM instruction. Rebasing cannot repair a failed continuous proof. |
| `crates/snes/src/cpu_synchronous_executor/source_cpu/timing_probe.rs::RomCpuTimingProbe::{new, from_cold_cpu_executor, step, into_handoff, from_handoff}` | Small unmodified routine executions, source transaction totals, exact retained handoff | `new` rejects pending interrupts/active hardware and has no transferred exact APU owner. Use retained cold ownership for active gameplay; retain its event cursor through handoff. |
| `crates/snes/src/cpu_timeline.rs::{CpuMasterTimeline, CpuMasterTimestamp, CpuFieldTiming}` | Physical master clock, synchronous event cursor, odd short field, refresh and HDMA ordering; separate work from stalls | Raster estimates or `1364*262` multiplication cannot reconstruct physical elapsed time, pending events or interrupt jitter. |
| `crates/zelda3/src/zelda_rtl/rtl_native_exact_cpu.rs::{NativeExactCpuOwner::new, capture_checkpoint, rebase_translated_timing_state}`; `rom_timing_shadow_execution.rs::{advance_native_exact_cpu_host, NativeExactCpuOwner::advance_host_with_hook}` | Existing cold private shadow, contiguous host assertion, diagnostic read/write/instruction collectors and selected native-state trial rebases | Current entry requires `rom_startup_timing` and no installed semantic receipts. It is not wired to lagless replay. Trial rebase exclusions are context-specific, not a universal WRAM import policy. Borrow mechanisms in a dev adapter; do not enable this runtime owner as the lagless clock. |
| `crates/snes/examples/logical_frame_ledger.rs::{main, write_seed, handler_returned, TimingAudit}` | Source iteration cuts, pre/post-NMI images, JSR/JSL frame accounting, instruction/routine receipts, timing-owner decomposition | `write_seed` includes memory/PPU and a DMA sidecar, not CPU registers, full stack ownership, timeline or exact APU checkpoint. Existing profiler accumulates selected rows in maps; full-route instruction collection needs streaming. |
| `zelda3-bin/src/logical_replay.rs::{seed_game, run_logical_replay}` | Native iteration driver, source RNG/APU answer streams, periodic seeds, actual main boundaries, routine and block companion CSVs | Seeds can be deferred and some sequences remain divergent. Preserve current seeding policy and log actual reseeds. Do not force fresh seeds to parallelize: the sharded-census audit found changed predictors. |
| `crates/zelda3/src/cycle_ledger.rs::{BlockMarkers::{block,taken,crossing,retire}, charge, routine, profile_iteration, profile_main_begin, profile_main_end, take_block_profile, take_routine_profile}` | Observe actual charged work, branch/page extras, scopes, unscoped charges and muting | Current block profile aggregates `(routine, PC, kind)`. It loses event order, call instance, entry widths and native site. Legacy scalars and separate VWF work are not block-resolved. |
| Same file: `routine_continuation`, `tail_routine`, `with_profile_call_path`, `assert_rom_marker_work` | Preserve real call-frame attribution across translated helpers/JMP tails; unmodified-ROM fixture comparisons | Existing fixture trail is test-only and checks scoped instruction totals. It is not route-wide retirement equivalence. |
| `scripts/{rom_block_entries.json,rom_block_costs.py}`; `crates/zelda3/src/rom_block_costs.rs::BLOCKS` | ROM-bound declared entry ranges/M/X and generated static costs, taken-branch/indexed-read PC metadata | Entries are bounds, not a complete call graph or native-site registry. Static decoder rejects unsupported widths, indirect dispatch and unresolved costs; never fill such gaps by guessing. |
| `scripts/{routine_coverage_diff.py,rom_block_worklist.py}` | Strict iteration-set and exclusive-total reconciliation; observed control-flow block partition; negative gaps | Routine differences remain ownership hypotheses. `rom_block_worklist.rank` explicitly leaves native M/X ambiguity unresolved and cannot block-resolve scalars. |
| `scripts/{rom_cpu_profile_summary.py,cycle_ledger_check.py}` | Existing profile inspection and focused diagnostics | Host slack joins, partial-plan treatment and skipped dispatch/root checks in `cycle_ledger_check` preclude a completeness certificate. Do not inherit its exclusions. |

The source receipts used by route ranking are **unmodified-ROM CPU executions**,
not inferred prices. `logical_frame_ledger` with `ZELDA3_LEDGER_PROFILE`,
`ZELDA3_LEDGER_BLOCK_PROFILE=1` and `ZELDA3_LEDGER_TIMING_AUDIT=1` records
routine, per-PC/M/X and owner-specific timing data. Its profiler uses the
pre-instruction SP and separates interrupt entry after an opcode. Main starts
at `$00:8038`, ends after `$00:805D`, and is labeled `$00:8034`; interrupts,
polygon work and idle spins have separate ownership. Reuse that definition.
Route 2's admitted marker checks covered 2,104 instruction-receipt iterations;
that subset must not silently become whole-route instruction coverage.

## Alignment and state ownership

### Two explicitly different lanes

**Source-continuous reference:** Boot the cold CPU once from the bound ROM and
initial SRAM, replay original host inputs, and retain the full machine. Read
full-v2 to verify each source iteration's index, opening/closing host, input,
RNG/APU consumption and pre/post-NMI images. Native continues through the
existing logical-replay driver. Join logical iteration IDs, not host callback
numbers: one main iteration may span several NMIs. Drain/discard source PCM
as required to bound memory; never render frames. No external Snes9x execution
is needed: `Snes9xColdCpuExecutor` is an in-repo CPU implementation.

**Native-seeded diagnostic:** At an explicitly proved source boundary, fork
the retained source checkpoint, import only native-owned memory regions and
the verified program-register state, and execute the next routine/iteration.
Keep the independent continuous source alongside it. Label results
`native_seeded`, include copied ranges and before/after differences, and
compare path/output/work against the continuous reference. This answers
"what ROM work corresponds to this native state?" only for validated slices.
It cannot certify original-route work after the native path has diverged.

Do **not** seed a fresh interpreter from native WRAM every iteration. Native
locals need not occupy the ROM stack; source-private DP/decode temporaries and
translated asset representations can differ. Existing
`rebase_translated_timing_state` demonstrates preserving stack, DP, ROM text
pointers and in-flight decode scratch. Each exclusion needs an explicit owner
and fixture. Never copy a source shadow's writes into the game.

### State required at each entry/resume

| Domain | Retained or captured state |
| --- | --- |
| CPU | PC/PBR, A/X/Y, SP and live stack bytes/return frames, DP/DBR, all status bits including M/X/E, WAI/STP, pending NMI/IRQ and acceptance deadlines; main/poly stack identity |
| Memory | Full 128 KiB WRAM and 8 KiB SRAM with write witnesses; native `wram_image()` at semantic boundaries; ownership of locals, scratch and published versus working generations |
| Hardware | Physical timestamp, odd field/refresh phase and processed-event cursor, MEMSEL, open bus, interrupt enables/timers, joypad serial/auto-read latches, PPU register/read-latch state and VRAM/CGRAM/OAM where read or DMA-visible |
| DMA | Eight channels, enable masks, live HDMA table/indirect cursors, line counters, pending/busy transfers and event phase; register setup alone is insufficient |
| RNG | Source RNG state and every draw including result/carry, source PC and ordinal; native ledger-fed draw cursor and extras/unconsumed counters |
| APU | Both port directions, ordered timestamped accesses/answers and command writes, upload/ack state; exact SMP/DSP coroutine plus APU clock for the continuous source lane |
| Native audit | Iteration/main begin/end, dynamic routine/site stack, object/sprite slot/state, continuation identity, retained locals/publication stage, marker sequence and mute status |

The census lane uses its existing recorded RNG/APU answers for native play.
Validate every consumed ordinal and source access; identical final bytes do
not establish identical read sequences. A separate live-APU experiment can
follow later, with its own manifest. Never substitute a final port byte for
the sequence of reads inside a song-upload/poll loop.

At each common entry/return and every suspension/publication, compare the
declared semantic outputs and branch-relevant inputs. Keep raw WRAM differences
alongside a versioned ownership map for source-private bytes; never silently
ignore differences or widen existing replay classifiers. At full-v2 boundaries,
also check source image equality and consumed event counts. A native atomic
publication may have no matching intermediate source snapshot; mark that
boundary `unmapped` until a retirement witness exists.

On first mismatch, emit both PCs/call stacks, register/status/stack deltas,
first differing memory writes and last writers, DMA/clock/interrupt state,
RNG/APU ordinal, native site, continuation and seed lineage. Keep a bounded
prelude trace and stop pricing admission for that slice. Continuing diagnostics
is allowed only with an explicit invalid interval; next seed creates a new
lineage rather than extending a clean prefix. Unsupported instruction/hardware,
step/time limit, missing input and incomplete final iteration are failures or
enumerated unmeasured intervals, never zero work.

## Attribution and exact conservation

### Pair retirements rather than routine names

Add a dev-only ordered observer, outside pricing semantics. For each native
marker retain iteration, sequence, block/extra PC, clocks, source routine path,
dynamic call ID, stable site ID plus `file:line:column`, entry kind and
continuation stage. Capture the site at the marker caller (for example by
propagating `#[track_caller]` through marker helpers); recording only
`retire`'s own location is useless. Uncharged native entries also need
observation so a missing marker has an actionable destination. A checked-in
dev mapping can relate source call edges/return PCs to these sites; multiple
candidates remain explicit until validated. Debug metadata must not affect
routine scope creation, `charge`, branch outcomes or scheduling.

Decode source instructions with actual entry M/X/DP/DBR and bus state. Retain
raw PC and normalize only proven LoROM mirrors. Partition at executed entry,
call/return, branch, width change, computed dispatch destination, interrupt,
and native publication/resume cut. Expand each generated static block to its
owned instruction PCs; pair taken/page extras with the actual branch/access.
Check dynamic timing against generated assumptions, including indexed indirect
base-low page crossing, MEMSEL and nonzero DP. Unsupported assumptions become
specific residuals, never an averaged correction.

Track JSR/JSL return frames, JML/JMP inherited ownership, RTS dispatch/pulled
frames, double returns, interrupts and polygon stacks separately. Match within
the same logical call and entry partition, preserving occurrence order. A
multiset match alone can hide a duplicate prefix followed by an omitted suffix.
Allow reordering only under a named, tested semantic mapping; otherwise report
path mismatch. A marker block spanning an interruption needs a split/witness
before its work can be distributed over retirement slices.

Legacy `charge(N)` gets its native site and total, but remains
`scalar_unmapped` until its instruction set is proved disjoint. VWF, raster
spin waits, uploads and DMA need explicit domains. Absence of a block marker
is **not** proof of uncharged instructions. Match known scalar ownership first
or withhold the affected candidate. Never add an inclusive routine difference
on top of priced children.

### Module09 is the first partition acceptance case

Preserve the [Route 2 entry audit](cost-pacing.md#module09-entry-partition-no-price-admitted).
Its source candidate ranges are `$02:A475..A47D` selector/call,
`$A47E..A4B4` scroll-save/Sprite_Main call, and `$A4B5..A4CC`
restore/LinkOam/HUD calls. Calls own their opcode only.

`crates/zelda3/src/overworld.rs::{publish_module09_link_oam_prefix,
complete_module09_after_sprite_main,
complete_module09_overworld_after_resumed_sprite_main}` exposes the ambiguity:
the prefix restores four locals; generic suspension can retain
`caller.link_oam=None`, also the fresh value; completion restores them again.
Final state equality does not distinguish publication from source retirement.
Native `OverworldModule`/`OverworldCaller` already retire these source opcodes;
LinkBody/LinkOam/HUD suffixes can bypass the translated completion helper.

The auditor can expose the exact PCs and duplicate/missing site events. It
does **not automatically solve** this lost-state distinction. Its dev adapter
must retain a source-call identity and monotonic retired-instruction cursor
independently of `link_oam`, with separate publication observations. Fixtures
must cover complete translated, native caller, partial-scroll (including final
vertical stores and call), generic LinkOam, stair-specific progress, LinkBody,
HUD and repeated resume handoffs. Every source opcode maps once to a retirement
site or a named gap; re-publication maps to zero additional source retirement.
Where current runtime state cannot prove the stage, leave
`ambiguous_retirement` and propose a later source-owned continuation change.
No blanket suffix or complete-entry-only price is admitted by this design.

Apply the same rule to slot dispatch: separate already priced entry/timer,
active tail, local-dispatch head/pulled-frame tail and selected handler body.
Slot/state/call identity must survive timer resumes and skipped/clipped tails.
An extra whole-slot price cannot be inferred from the dispatcher parent gap.

### Completeness contract

For iteration `i`, in **master clocks** (not 65816 instruction counts), define
`S_i` as source main instruction transactions, `N_i` as native main instruction
charges including instruction-classified scalar/VWF work, `P_i` as matched
source work and `Q_i` as its paired native charges. Keep unmatched source work
`U_i`, unmatched native work `O_i`, and signed paired cost error `E_i=P_i-Q_i`:

```text
S_i = P_i + U_i
N_i = Q_i + O_i
S_i - N_i = U_i - O_i + E_i
```

Both unmatched sides are nonnegative and retain all rows. `E_i` retains signed
per-block differences, so cancellation cannot hide overcharging. Scalar or
ambiguous candidates may be numerically measured yet unresolved in ownership;
list them on the corresponding side with status, rather than guessing a pair.
Declare exact coverage only with valid alignment, one-to-one ownership,
`U_i=O_i=E_i=0` **and every paired row equal**, not just zero net difference.

Reconcile native observer events against the independent main counter, with a
separate bridge for VWF, waits, DMA and other non-instruction charges. Every
charge, including owner zero, must enter exactly one domain; an unclassified
scalar stays `native_domain_unknown`. Reconcile source transactions against
independent routine and timing-audit totals. Preserve interrupt entry separately
when accepted after an opcode. Opening/lag NMI, IRQ, polygon, DMA, HDMA,
refresh, idle and polling/waits get independent measured totals/residuals.
Source polling instructions must be moved once from instruction work to the
wait comparison domain if that normalization is used, with raw totals retained.

Thus "native charged == ROM cycles" always names its domain. Main instruction
equality alone is not physical elapsed equality. Emit a second reconciliation
for active wall time, preserving endpoint refresh uncertainty and missing
hardware ownership as residuals. Do not change P2 to make it equal. Route
completion requires all 1,473,123 unique ordered iteration rows, verified EOF,
zero absent intervals, and sums equal to independent counters. An explicitly
enumerated residual report is useful completion of an audit, not a timing pass.

## Output and pricing workflow

Interface contract: a separate `rom-shadow-audit` development binary. The
M0/M1 subset is implemented below; ordered block pairing remains M2. Accept
explicit read-only ROM, full-v2 and input/
initial-SRAM paths and one fresh output root. Manifest binds commit/dirty diff,
binary/features, ROM SHA-256, ledger/input/SRAM identities, mapping/table hashes,
seed policy and actual reseeds, source checkpoint schema, iteration cuts,
clock units, excluded domains and completion status. Reject changed identities.

| Artifact | Required contents |
| --- | --- |
| `iterations.jsonl.zst` | One summary per iteration: source/native totals by domain, matched/unmatched/error terms, alignment, hosts, seed lineage and residual-row references |
| `blocks.jsonl.zst` | Streamed `(iteration, source call, PC/end, M/X, native site, entry/stage, counts, source_master, native_master, signed_delta, status)`; retain unpriced/overpriced rows and unknown sites |
| `residuals.jsonl.zst` | Stable ID, domain/reason, measured clocks or `null` if unmeasured, iteration range, source PCs and native site candidates, ambiguity and diagnostic witness references |
| `worklist.json` | Disjoint validated positive gaps and negative-price repairs; frequency, total clocks, call/entry variants, first witness, required fixture and affected missing-hold IDs |
| `coverage.json` | Exact row/EOF and counter checks, paired and unpaired work, missing/invalid intervals, domain residuals and valid-prefix boundary; pass status cannot omit unknown rows |
| `witnesses/` | Bounded compressed instruction/marker preludes, register/write/port evidence and ROM-free checkpoint metadata for selected failures |

Illustrative row shape (values deliberately omitted):

```json
{"iteration": 472388, "source_call": "call-id", "pc": "02a4b5",
 "native_site": "overworld.rs::complete_module09_after_sprite_main/restore",
 "entry": "resumed", "retirement_stage": "unknown",
 "source_master": null, "native_master": null,
 "status": "ambiguous_retirement", "witness": "residual-id"}
```

A pricing batch freezes the manifest/cohort, selects the largest **validated
exclusive instruction gaps**, and inspects exact source PCs, entry variants
and proposed native sites. It adds static generated markers in a separate
batch with ROM fixtures, once-only resume checks and signed per-iteration
counter proofs. Re-run the auditor: selected unmatched work must disappear
without new duplicate/overcharge/path residuals. Default accounting and replay
checks remain separate acceptance gates.

Hold inverses remain a secondary impact estimate, including every multi-hold
threshold and fixed source context. Union affected IDs to avoid overlapping
scores. Barrier/HDMA changes can invalidate fixed-context predictions (Route 1
retains two such cases); flag and measure them. Frames flipped validate the
result afterward; they no longer define what instructions deserve a price.

## Cost and build isolation

No auditor throughput measurement is claimed. A published unsharded native
full-v2 control took about **55 minutes** on a loaded machine
([sharded-census measurements](cost-pacing.md#full-v2-verification-and-wall-times));
it is a baseline reference, not a current isolated benchmark. Source execution
must cover original host calls, including lag, idle, intro and uploads, even
when only missing gameplay rows are retained.

M1 measured **204,058,248 source opcodes over 16,000 original hosts**.
The final instrumented prefix averaged **1.395 million instructions/s**;
the measured native prefix took **13.965 s for 14,000 iterations**.
Extrapolating the observed 13,959 source opcodes per completed iteration to
1,473,123 iterations gives 20.564 billion opcodes: **4.095 source hours +
0.408 native hours**, or approximately **4.5 hours serial**, before any
route-dependent change in work mix or future M2 detail output. This replaces
the unmeasured runtime guess. It is a prefix-based estimate on a loaded
machine, not a full-route bound or permission to start a census. The five
measured windows, output overhead and limits are in [M1 result](#m1-result).
Runtime formula remains `T ≈ T_native + source_opcodes / measured_rate + output_time`.

Keep one source owner and one native game; optionally one bounded diagnostic
fork. Budget **under 512 MiB additional audit memory**, with a separately
measured whole-process peak RSS target below **2 GiB**. Stream per iteration;
cap ring buffers and call depth; intern sites/PCs; never retain route-wide
instruction maps. Copying 128 KiB every iteration alone moves about **193 GB**
decimal, so prefer retained owners, incremental hashes and selected snapshots.

At 128 bytes per iteration, raw summaries cost about **189 MB**. Twenty
16-byte aggregate detail records per iteration cost about **471 MB** before
compression. Hundreds of records per iteration exceed the **3 GB** workspace
budget even before traces/builds. Therefore stream all summary/conservation
rows, aggregate exact-match detail, retain complete mismatch/residual rows,
and regenerate detailed matched traces from identity-bound checkpoints on
demand. Trace every opcode only in bounded witnesses; even 16 bytes/opcode
would be 118–354 GB under the assumptions above. Target **0.5–1.5 GB** audit
outputs, reserve space for builds, and stop with `incomplete_disk_budget`
before 3 GB. Never truncate evidence silently or treat retention sampling as
execution sampling. Account for both compressed bytes and temporary files.

Prefer a new **non-default** `rom-shadow-audit` feature in a separate dev
package, with explicit `required-features` on the binary and dev observer APIs
gated throughout. It can depend on existing `parity-debug` and `snes/rom-timing`
without enabling the game's `rom_startup_timing`. `parity-debug` alone is
default-on today, so it is too broad for accidental ROM execution prevention.
The package's explicit ROM loader owns the only audit ROM read. Product
`BlockMarkers` continue reading only `rom_block_costs.rs`; there is no dynamic
price lookup, source-cycle feedback, fallback shadow or ROM-path environment
switch in the product. Existing timing modules are not a reason to expand the
product's ROM dependency.

Packaging already uses `--no-default-features`
(`scripts/package_macos.sh`). Later acceptance must build that configuration,
verify the auditor/observer APIs are absent, and exercise packaged lagless play
without a ROM file while comparing timing counters to its normal baseline.
Default dev builds with the new feature off must likewise have no new ROM
reads. Keep any runtime-independent dev rebase helpers behind the new gate;
new debug switches use `crate::debug_env`, consistent with `CLAUDE.md`.

## Stages, risks and feasibility decision

No execution probe was run in this batch. A WRAM transplant alone would not
answer the key question; no inspected CLI joins a lagless iteration to a proved
ROM register/stack/hardware checkpoint. Building that adapter is substantive
M1 work. Existing fixtures and CPU checkpoint APIs establish reusable mechanisms,
not successful native-seeded whole-gameplay iteration equality.

Estimates below concern how many of the **2,919 missing holds become tractable
to investigate**, not automatic recoveries. They are judgment ranges, overlap,
and depend on M1 alignment results. No stage earns recovered frames without a
later independently checked pricing change.

| Milestone | Deliverable and acceptance | Estimated tractability |
| --- | --- | --- |
| M0: freeze contracts (1–2 engineering days) | Bind identities, iteration/domain cuts, existing receipts and residual schema. Reproduce source/native exclusive counter checks on retained samples; reject absent/duplicate rows. | 0 newly resolved holds; prevents false admission. |
| M1: valid shadow handoff (3–5 days) | CPU-only dev harness, one cold retained source and native logical replay. Prove one ordinary Module09 and one dungeon iteration with entry registers/live stack/hardware state, source path and CPU count equal to an independent source receipt. Fork at the proved entry, import native-owned regions, require the same return/path/output/work or report the first precise divergence. Repeat for a resumed iteration and restore/re-execute checkpoint. Source trace/counters and native predictors must be identical with observer off/on. Benchmark the five window types above. | 100–400 holds (3–14%) in verified ordinary paths may become actionable; zero guaranteed. Failure to align one ordinary iteration is a stop/go result. |
| M2: ordered marker pairing (4–7 days) | Site/call/width-aware observer, scalar/VWF accounting and per-iteration equations. Cover priced child aliases, page/branch extras, double returns and local dispatch; deliberately missing/duplicate/wrong-cost test events must produce exact signed PC/site residuals. No net-cancellation pass. | 600–1,200 holds (21–41%) across straightforward body/caller gaps; includes M1. |
| M3: resume partitions (5–10 days) | Module09 entry matrix and slot timer/body resumes, source retirement cursor versus publication. Every opcode has one owner or a specific ambiguity; replayed publication never retires source work twice. A future runtime continuation fix, if necessary, is a separate reviewed batch before pricing. | Additional 200–600 holds (7–21%) may become investigable. Route 1's 314 routing score is context, not a Module09 recovery bound. |
| M4: full census residual accounting (3–5 days plus measured run time) | Stream every full-v2 iteration in one serial pass, unchanged native seeding/predictors, independent source image/event checks, EOF and all conservation equations. Enumerate every remaining domain/path/ownership residual, with no null-as-zero. Verify bounded RSS/disk and deterministic repeat of selected checkpoint windows. | Cumulatively 1,500–2,300 holds (51–79%) could have concrete worklists; difficult upload/polygon/timing-defined paths remain residuals. |
| M5: hard owners and product exclusion (open-ended, initially 1–2 weeks) | Add supported adapters for upload/poll, VWF, polygon, IRQ/HDMA domains; retain failed alignments. Separate feature/package checks, ROM-free packaged timing check and observer noninterference receipts. Full exact coverage requires zero unresolved work/path rows; otherwise publish the complete residual ledger. | Potentially the remaining 619–1,419 holds become diagnosable. Some require behavioral/ownership repairs rather than pricing. No claim of 100% recoverability. |

Principal risks are source/native path disagreement despite equal endpoint
WRAM, missing intermediate native retirement state, scalar work that cannot be
mapped uniquely, and shared-tail/width aliases. The auditor cannot manufacture
an exact native site where the port has erased that distinction. Return site
candidates and required stage evidence, rather than distributing work by a
heuristic. A successful source CPU run also shares emulator code with existing
receipts; it is not independent hardware or external A/V proof.

The first useful success is an exact instruction residual with a proven entry
and an actionable native site. The final useful success is a route-wide
conservation report with every remaining unknown named. Neither changes pricing,
pacing, classifiers or runtime behavior in this documentation batch.

## Verification of this batch

Both unchanged ordinary binary configurations build successfully, with zero
warnings/errors: default features (**2m 00s**) and `--no-default-features`
(**2m 28s**). Commands were `cargo build --offline --profile parity -p
zelda3-bin`, adding `--no-default-features` for the second build, with
`CARGO_TARGET_DIR=target/shadowplan/build`, `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_PARITY_DEBUG=0`. Logs are
`target/shadowplan/build-{default,reduced}.log`.

`git diff --check` passes; the added document also passes a no-index whitespace
check against `/dev/null`. Scope verification finds this document as the only
working-tree change, with HEAD still `1f93f138`. `target/shadowplan` occupies
about **1.0 GiB**, below 3 GB. No feasibility execution, game/GPU run, tests,
protected-cache/shared-ledger write, runtime/pricing edit, cargo fmt, commit
or push was performed. This is a verified design stop; M1 execution feasibility
and all implementation remain future work.


## M1 result

**M2 verdict: STOP.** M0 is implemented; M1 establishes two ordinary diagnostic
handoffs and a precise unmapped resumed handoff. It does not establish a
route-wide native/source join. No pricing, pacing, classifiers, P2, markers,
block tables, seeding policy or product behavior changed. HEAD remains
`24b4fd3acce814dcd846c692c639d2a08ca9fac5`, branch `codex/shadow-m1`;
there is no commit or push.

### Implementation and isolation

`crates/rom-shadow-audit` contains the separate CPU-only development binary,
with empty default features and `required-features = ["rom-shadow-audit"]`.
Its explicit loader owns the audit ROM reads. The Zelda library and game
binary expose a separate, non-default feature of that name. All native hooks
are under that gate; `parity-debug` alone does not enable them. Default and
reduced product builds therefore have no new ROM access or observer API.
The default binary contains none of the new audit-switch strings.

The native hook observes charge totals at the existing main cuts, including
owner-zero work; separate VWF remains separate. It adds no marker, scope,
charge or scheduling decision. `ZELDA3_SHADOW_AUDIT_COUNTERS` selects a fresh
JSONL output, and `ZELDA3_SHADOW_AUDIT_ENTRIES` selects an existing fresh
snapshot directory. Both switches use `crate::debug_env`. Logical replay
supplies the actual iteration identity without altering its seed decisions.
Entry images are captured at native main begin after the opening handler.

M0 binds the ROM, SRAM, input, every full-v2 file, executable, source executor,
import-map/contract and static table identities. The manifest records commit,
dirty-source hash, enabled features, checkpoint schema 9, main cuts, master
clock units, domains and seed policy. Native evidence binds its executable,
predictor, stdout and actual seeded segment records. Verification rejects a
changed bound input. Retained source/native exclusive CSVs are checked on
**all 14,000 iterations 0..13,999**, including owner zero and VWF. Absent
iterations, absent totals, duplicate/mirrored routine rows and unequal
exclusive sums are rejected. Signed routine differences remain ownership
residuals, including negative differences; they are not paired prices.
Unknown domain clocks remain `null`. Tests explicitly reject net cancellation
as an exact pass.

### Retained source and ordinary forks

One reset-driven source owner executes the original host inputs, with frame
capture disabled and PCM drained every host. The final run executes
**16,000 original hosts / 204,058,248 opcodes**. It verifies **14,618 ordered
full-v2 rows**, exact pre/post-NMI WRAM images, opening/closing hosts, input,
joypad, RNG draws/carry, APU read/write sequences, lag NMIs and IRQ counts.
The unexecuted/unclosed interval **14,618..1,473,122** is explicit; EOF is
unverified and coverage remains false.

The unchanged `logical_frame_ledger` example supplies independent source
receipts: **14,001 main instruction totals and 14,001 structural instruction
totals match exactly**. Independent chronological traces also match every
PC, start/end timestamp, raster line and interrupt flag on host 3,740
(**12,759 instructions**) and host 4,938 (**12,818 instructions**). These host
traces include non-main work; the main counts below exclude it.

| Ordinary entry | Iteration / source host | Main instructions | Source master clocks | Native-seeded fork |
| --- | ---: | ---: | ---: | --- |
| Dungeon `07/00`, `$00:8038` | 3,262 / 3,740 | 3,035 | 81,252 | Same return, path, transactions, accesses, output and hardware checkpoint |
| Module09 `09/00`, `$00:8038` | 4,326 / 4,938 | 4,394 | 115,840 | Same return, path, transactions, accesses, output and hardware checkpoint |

Each witness retains all program registers/status/pending flags, the full
stack page, timestamp, raster, DMA state and a hash of the complete retained
hardware/APU checkpoint. Raw checkpoint serialization stays in memory because
schema 9 contains the ROM. Exact restore/re-execute checks compare every
receipt field for 256 instructions and the final complete checkpoint hash.
They pass at both ordinary entries and at the resumed boundary below.

`ORDINARY_PRIVATE_V1` is a fixed ordinary-entry import map derived from the
existing replay representation boundaries. It preserves direct page/stack,
source-private decoder and relocated representations; the outdoor upload
range is excluded only for Module09, leaving dungeon door state imported.
Its exhaustive byte-ownership test rejects overlap or uncovered bytes. WRAM
ranges and SRAM are copied only into the private source fork; retained source
registers and all hardware owners remain intact. Every raw entry difference
and copied range is retained. **Both samples have zero differing bytes in the
imported ranges**: these are useful aligned controls, not evidence that an
arbitrary changed native state will follow the same ROM path.

Only one diagnostic CPU exists at a time. A bounded reference witness is
collected and its diagnostic owner dropped before creating the imported fork;
the continuous source is never modified. Both fork paths/counts independently
match the subsequently observed continuous source summaries. The map is
admitted only at ordinary `07/00` or `09/00` `$00:8038`; it deliberately
rejects resumed PCs and other substages.

### Precise resumed stop

At **iteration 1,916, source host 2,294, module `07/0F`, PC `$0D:A95B`**, the
source resumes after an interrupt handler. Its retained registers, live stack,
hardware state and exact 256-instruction restore/re-execute are captured.
The native replay's iteration-entry image and endpoint cannot establish the
CPU registers, live locals or retired-opcode cursor at this intermediate
boundary. The result is **`native_resume_unmapped` / `unmapped_resume_handoff`**,
with unmeasured clocks, not an aligned zero-cost row. No fresh-entry image is
transplanted there, no alignment is forced, and no resumed fork is admitted.

This is the required M1 feasibility stop. The ordinary controls do not supply
that missing continuation proof. A later dev adapter must expose and validate
this intermediate native stage before M2 ordered pairing proceeds. The
Module09 publication-versus-retirement ambiguity described above remains
unresolved; these two ordinary controls do not waive it. No validated price
gaps or negative-price repairs are emitted into the worklist.

### Measured throughput and budget

These are instrumented CPU-only measurements from the final retained run on
the loaded machine. Classification observes actual source PCs/state: reset
hosts 0..80; polygon-thread activity; APU upload PCs `$00:8888..8912`; Module06
load hosts 11,477..11,537; ordinary `07/00` or `09/00` otherwise. Polygon and
ordinary groups contain multiple windows. Upload hosts are 11,538..11,560.

| Window | Source hosts | Executed opcodes | Instructions/s | Source hosts/s |
| --- | ---: | ---: | ---: | ---: |
| Reset | 81 | 1,160,504 | 800,176 | 55.85 |
| Ordinary gameplay | 9,132 | 115,862,792 | 1,452,062 | 114.45 |
| Long load | 61 | 684,299 | 1,168,113 | 104.13 |
| Polygon | 742 | 9,068,667 | 973,757 | 79.67 |
| APU upload | 23 | 325,464 | 1,568,288 | 110.83 |

The remaining 5,961 hosts execute 76,956,522 opcodes and remain in `other`;
they are included in the overall rate and runtime projection. Source execution
and observation total **146.279 s**; initial identity freezing, final
rechecking and output add **20.243 s**, for **166.522 s** process wall time. The **4.5-hour** serial projection above uses this entire prefix,
including reset, idle and upload work, plus the measured native replay rate.
Later route work mix, contention and M2 output cost remain unmeasured; this
estimate is not a claimed full-census wall time. No census is scheduled while
M2 is stopped.

Whole-process peak RSS and exact commands are retained in
`target/shadow-m1/source-admitted-process.json` and `native-final-runs.json`.
The final source peak is below 100 MiB; native peak is **368,836,608 bytes**,
well below 2 GiB. These are whole-process measurements, not an isolated proof
of incremental allocator overhead. Witnesses are bounded (64-step failure
prelude, at most eight selected hosts / 200,000 trace rows, 100,000 fork-step
cap). Outputs stream per iteration. A source run checks workspace bytes every 1,000 hosts and emits
`incomplete_disk_budget` at its 2.8-billion-byte guard, reserving room below
the 3 GB limit. No route-wide instruction map or ROM-bearing checkpoint is persisted.

### Verification and retained evidence

The final ordinary product builds are warning-free in default,
`--no-default-features`, and `--features rom-shadow-audit` configurations.
The auditor builds warning-free both enabled and disabled. **Ten new-package
tests** and the **single filtered Zelda observer test** pass. Baseline Zelda
lib-test dead-code warnings are separate from ordinary build results. No full
library suite or GPU test ran. Both fresh baseline and final default
**200,000-frame ledgers match every record and EOF** against the retained
schema-3 reference.

On iterations 0..13,999, the original feature-off product, final feature build
with observers off, and final feature build with observers on produce
**byte-identical predictor CSVs, routine CSVs and stdout**. All 14,000 observed
charge sums equal the independent main-counter deltas; adding separate VWF
reproduces every predictor main total. Source totals and the two chronological
traces also match the independent observer/control path described above.
`git diff --check` passes.

Final evidence is under **`target/shadow-m1/`**, particularly:

- `verification.json`, `observer-noninterference.json`, build/test/default logs;
- `source-admitted/{manifest,coverage,benchmarks,worklist}.json` and compressed
  iteration, entry, residual and bounded source-path witnesses;
- `m0-verified/` for the strict retained exclusive-counter reproduction;
- compressed independent profiles/traces and compressed native entry images,
  predictors and actual seeded-segment stdout, with identity/pruning receipts.

The `source` command's zero exit means a completed diagnostic report, not an
alignment pass; inspect `coverage.json` and its explicit **STOP / false**.
The `verify MANIFEST` command checks bound files; `receipts SOURCE_CSV
NATIVE_CSV LO HI FRESH_OUT` independently reproduces retained counters and
rejects missing/duplicate rows. CSV inputs may be zstd-compressed.
Build the separate binary with `cargo build --offline --profile parity -p
rom-shadow-audit --features rom-shadow-audit`; its source interface is
`source ROM SRAM INPUT FULL_V2 HOSTS FRESH_OUT [NATIVE_ENTRIES [WITNESS_HOSTS]]`.
Native entry inputs are compressed WRAM/SRAM images plus bound provenance,
not standalone source checkpoints.

Superseded runs and raw RAM/profile/trace artifacts are deleted after bounded
compressed retention. The workspace remains below **3 GB**. No GPU run,
`--video`, cached-av, external Snes9x process, oracle-cache/shared-ledger write,
full library suite, cargo fmt, commit or push was performed. The source CPU
and independent receipts share emulator code; this is CPU audit evidence,
not independent hardware or live exact A/V acceptance.

## M2 (ordinary entries) result

**Pricing verdict: STOP. M2 is a partial implementation checkpoint, not an
accepted ordered source/native join.** The full native census and a longer
verified source prefix are complete. Ordered observation, instruction
partitions, bounded counter reconciliation and the pairing kernel are
implemented. The live source-call/block-to-native-site bridge is still
unfinished; the kernel has not been applied to a proved live mapping. No price
is admitted or applied. This result does not establish that ordinary M2 is
infeasible.

Work is on `codex/shadow-m2`, based on `8e34ebe98462f218ba4fca619dfb20c081ff67e5`
(main plus Route pricing 5 and M0/M1). No other worktree was modified. No
pricing, pacing, classifier, P2, marker semantics, generated block table,
seeding decision or ordinary product behavior changed. New observation and
accounting remain behind the non-default `rom-shadow-audit` feature; serde_json
and zstd runtime dependencies are optional under that feature.

### Entry-kind census

The fresh ROM-lane native replay uses full-v2, `--seed-every 2000`, original
recorded APU answers and `--pacing-predictor`, without video. It completes all
**1,473,123 ordered iterations**, reporting no WRAM divergence. There are
**2,787 missing-hold iterations / 2,820 missing holds**. A hold deficit is
`max(lag_nmis - native_held, 0)`; excess holds do not cancel deficits.

One reset-driven, serial source owner executes **60,000 original hosts /
755,156,459 opcodes** and verifies **53,706 iterations (0..53,705)** against
full-v2 images and ordered input/RNG/APU/interrupt receipts. Coverage remains
false; **53,706..1,473,122 (1,419,417 iterations) and source EOF are unverified**.
The source coverage timer records 1,600.110 seconds, including observation and
final identity verification, on the loaded machine. It is not an isolated
throughput measurement or a full-route runtime claim. The existing 2.8-billion
byte source guard checks every 1,000 hosts. No source sharding or new seeding
policy was used.

Entry classification observes release into `$00:8038`, after any `$00:8034`
LDA / `$00:8036` BEQ wait-loop instructions. An RTI returning into that wait
loop is not itself classified as a resumed main entry. Only M1's `07/00` and
`09/00` import-map entries are eligible; no further module/substage is admitted.
Predictor module fields describe the closing image and are not used as entry
classification proxies.

| Source entry kind | Iterations | Missing iterations | Missing holds |
| --- | ---: | ---: | ---: |
| Ordinary admitted `07/00`, `$00:8038` | 22,608 | 0 | 0 |
| Ordinary admitted `09/00`, `$00:8038` | 14,494 | 29 | 29 |
| Ordinary unadmitted, module/substage retained | 16,013 | 26 | 34 |
| Resumed entry, `unmapped_resume_handoff` | 591 | 1 | 1 |
| Unexecuted source suffix, entry kind unknown | 1,419,417 | 2,731 | 2,756 |
| **Total** | **1,473,123** | **2,787** | **2,820** |

Unadmitted ordinary entries with missing holds in the verified prefix are:

| Module/substage | Entry iterations | Missing iterations | Missing holds |
| --- | ---: | ---: | ---: |
| `00/00` | 1 | 1 | 1 |
| `00/01` | 25 | 7 | 7 |
| `00/07` | 70 | 5 | 5 |
| `01/00` | 1 | 1 | 9 |
| `06/00` | 8 | 2 | 2 |
| `07/02` | 1,726 | 4 | 4 |
| `07/0E` | 2,080 | 1 | 1 |
| `08/00` | 10 | 2 | 2 |
| `09/20` | 3 | 1 | 1 |
| `0E/03` | 460 | 2 | 2 |

The complete module/substage table, including zero-deficit states, is in
`target/shadow-m2/census-full/entry-kind-table.json`; individual missing IDs are
in its companion `missing-entry-kinds.jsonl.zst`. No entry-kind distribution
is inferred for the unexecuted suffix.

Entry eligibility is distinct from an intermediate retirement proof. M1's
iteration 1,916 enters ordinary **unadmitted** `07/0F` at host 2,293 and later
reaches the retained unmapped resumed handoff at host 2,294. Both observations
are retained. All **29 admitted missing-hold cases also encounter an internal
interrupt**. Their resumed phases remain explicit `unmapped_resume_handoff`
residuals; no fresh image is imported there.

### Ordered observation and conservation

The gated native sink records ordered scope entries/exits (including uncharged
scopes), dynamic native scope IDs/path, marker PC/kind, generated entry M/X
contract, actual caller `file:line:column`, and every unmuted charge, including
owner zero. Taken/page extras retain their kinds and unknown width where no
width witness exists. Scope IDs are explicitly **native scopes, not proved ROM
call IDs**. Scalars remain `scalar_unmapped`; VWF is reconciled separately
through the existing predictor counter. No scope, marker or charge is added
by the observer. Muted preview work remains absent.

`ZELDA3_SHADOW_AUDIT_EVENTS` selects a fresh output; a `.zst` suffix streams
compression directly. `ZELDA3_SHADOW_AUDIT_RANGE=LO-HI` is required, with an
optional `ZELDA3_SHADOW_AUDIT_ITERATIONS` comma-separated selection. The same
selection can bound native entry-image capture. All switches use
`crate::debug_env`. Thirty-one native entry-image pairs are retained: the two
M1 controls and the 29 missing cases. Capturing them is not a per-case imported
ROM-fork or source/native path proof; no new such proof is claimed.

The source stream retains **341,299 individual opcode partitions** for the 29
missing ordinary entries, with raw PC/opcode, actual M/X, DP/DB, stack positions
and interrupt cuts. Four bounded accounting windows reproduce their source
main counters and native charge counters, including the separate VWF bridge.
Every source opcode and native charge remains on its unmatched side:

```text
S = 8,946,084 = P (0) + U (8,946,084)
N = 8,520,516 = Q (0) + O (8,520,516)
S - N = 425,568 = U - O + E (0; no admitted pairs)
```

These equations hold per iteration, not just in aggregate. Every iteration's
alignment and exact-coverage flags remain false. Unpaired block errors remain
`null`; aggregate E=0 is the empty paired sum, not equal pricing. The **6,916
source instructions after the first interrupt** are separately marked
`unmapped_resume_handoff`, with source interruption/resume PCs and unmeasured
native continuation clocks. Earlier instructions still require a call/site
mapping; they are not silently admitted.

The pairing kernel requires proved call/occurrence, PC/kind, M/X, expected
native site and domain keys in occurrence order. Tests retain exact signed
PC/site rows for missing, duplicate, wrong-cost and wrong-site events, reject
width/call/domain aliases and reordered paths, and reject cancelling positive
and negative paired errors. Independent-counter tests reject a deleted source
instruction and a wrong observer total. This kernel is **not yet wired to a
validated live source-call/block expansion**. Priced-child aliases, local
dispatch/double returns, branch/page expansion and scalar instruction ownership
still need that bridge before a live price can be emitted.

### Worklist and pricing decision

`target/shadow-m2/worklist.json` contains **zero validated positive gaps, zero
validated negative repairs, zero admitted clocks and zero missing holds
covered by an admitted price**. Therefore both requested validated rankings
(by clocks and by missing holds covered) are empty. All native unmatched work
is retained; positive iteration differences do not establish that individual
blocks are free of overcharge.

For investigation only, the largest **unpaired iteration-total differences**
are iteration 35,306 (+20,996 clocks), 35,294 / 35,300 / 35,304 (+20,008 each),
and 50,775 (+18,906). Each corresponds to one missing hold. These are not
exclusive PC/site gaps and must not be copied into a price. No negative
iteration-total differences occur in these 29 rows; negative per-block errors
remain unknown until pairing is proved.

**STOP for pricing batches.** Continue ordinary M2 by implementing and proving
the source call/block-to-native-site bridge, preserving scalar ownership and
the named resume residuals, then applying the ordered kernel to those proved
keys. No runtime or pricing change is authorized by this worklist.

### Verification, retained evidence and limits

- All **16 auditor-package tests** and **three filtered Zelda observer tests**
  pass. No full library suite or GPU test ran.
- Default, `--no-default-features`, and feature-enabled ordinary builds, plus
  enabled/disabled auditor builds, are warning-free. Baseline lib-test
  dead-code warnings are separate from ordinary build results.
- On a **35,311-iteration replay**, with routine profiling on
  **35,290..35,310** around the first four admitted missing cases, default,
  feature-off and feature-on predictor/routine/block/object/sprite CSVs and
  stdout are byte-identical. All **53,706** ordinary-capture predictor rows
  also match the canonical default census prefix byte for byte; every native
  observed charge sum plus separate VWF matches its predictor total.
- The final default **200,000-frame schema-3 state ledger** matches every
  record and EOF against the retained reference. This is a CPU-only state
  check, not exact A/V acceptance. `git diff --check` passes.
- Source evidence is frozen under `source-prefix/`: the original manifest is
  unchanged; `manifest-frozen.json` and `manifest-relocation.json` bind
  byte-identical retained copies of the exact source-run executable and four
  audit source files before subsequent accounting-only edits. The frozen
  manifest verifies successfully.
- `ordinary-accounting-final/` retains per-iteration equations, separate
  source/native rows and interrupt residuals. `ordinary-events.jsonl.zst`,
  `ordinary-native-entries/`, `final-native-processes.json`,
  `final-observer-noninterference.json`, `worklist.json` and build/test logs
  supply the remaining evidence. Accounting windows cap retained source
  partitions at 200,000; they stream compressed inputs and outputs.

The disk limit was **missed transiently**: an observed raw-event snapshot was
**3,032,240,128 bytes**, above the requested 3,000,000,000 bytes. This is not a
proved absolute peak. Completed raw files were compressed/pruned, and the
final observer now supports direct compressed streaming. The miss and
lossless retention identities are recorded in `compressed-retention.json`;
final retained size is recorded in `verification.json` below the limit.

No GPU run, `--video`, cached-av, external Snes9x process, protected-cache or
shared-ledger write, cargo fmt, commit or push was performed. Source execution
and receipt checks share emulator code. Full source classification, live
ordered pairing and pricing admission remain incomplete.

## M2b result

**Pricing verdict: STOP. Continuous call reconstruction and a live ordered
marker subset are verified; complete instruction-to-site ownership and scalar
exclusion remain incomplete.** Work is on `codex/shadow-m2` at `b7cfeb65` plus
uncommitted auditor changes. The pricing worktree was not touched. No runtime
or pricing change, commit or push was made.

### Continuous lane and bridge rules

An internal NMI does not invalidate the retained continuous machine's main
work. Capture now includes every ordinary entry at `$00:8038`, independently
of the native-seeded fork import map. Entry classifications and the fork map
are unchanged. `unmapped_resume_handoff` remains a fork limitation; continuous
accounting records an interrupt cut and continues the same logical call.
Handler instructions and interrupt-entry transactions remain excluded from
main instruction work. No native state is imported at a resume.

The feature-gated `bridge ROM SOURCE_OUT PREDICTOR EVENTS FRESH_OUT` command
reconstructs JSR/JSL frames from actual targets and return PC/SP, checks RTS/RTL
pull widths, and retains JMP/JML tails under the inherited return frame.
Named tests cover local RTS dispatch, pulled-frame double returns, priced-child
interior partitions, native root fragments, and native paths projected onto
source ancestors. Shared-tail occurrence order follows executed alias entries,
not frame creation order. Repeated scope/block cardinalities must agree before
pairing; width, site and taken/page extras must also agree. Uncharged scopes
are retained.

The unchanged generated blocks expand into ROM opcode PCs, following REP/SEP
and checking actual M/X and DP. Each admitted source opcode is owned once.
Taken extras use the retired next PC across interrupt acceptance; absolute
indexed page extras use captured X/Y. Indexed-indirect pages without a pointer
witness, incompatible widths, unresolved aliases and reordered paths remain
unmatched. Native extra sites accompany the base marker site. Scalars and VWF
remain unmatched: numeric agreement is not an instruction-set exclusion proof.

### Verified prefix and ordered accounting

A fresh reset-driven pass executes **60,000 hosts / 755,156,459 opcodes** and
again verifies **53,706 iterations (0..53,705)**. Its decompressed summaries
and entry classifications are byte-identical to M2's retained prefix. The new
capture contains **4,285,899 main opcodes** for all **55 ordinary missing
iterations / 63 missing holds**. All 55 call-frame reconstructions succeed,
including **324 internal interrupt cuts**. No further prefix was attempted;
1,419,417 remaining iterations and source EOF are still unverified.

Native replay completes 53,706 iterations with **no WRAM divergence**. Its
predictor is byte-identical to the canonical default census prefix. Selected
observation retains **9,058,244 events**: 3,515,886 entries, the same number of
exits, 2,026,362 charges, and 55 begin/end pairs. An initial whole-window reader
stopped at its event guard without admitting prices. The final reader streams
compact records one iteration at a time, validating scope nesting and charge
paths, including zero charges. The direct final join exits zero in **74.839 s**,
with **418,562,048 bytes peak RSS** measured by macOS `getrusage`.

The kernel admits **24,535 equal block occurrences / 3,197,516 clocks**.
Independent counters reconcile in every iteration:

| Ordinary entry group | Iterations / missing holds | S | N | P = Q | U | O | S - N |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Admitted `09/00` | 29 / 29 | 8,946,084 | 8,520,516 | 995,682 | 7,950,402 | 7,524,834 | 425,568 |
| Unadmitted ordinary | 26 / 34 | 116,947,094 | 93,625,966 | 2,201,834 | 114,745,260 | 91,424,132 | 23,321,128 |
| **Total** | **55 / 63** | **125,893,178** | **102,146,482** | **3,197,516** | **122,695,662** | **98,948,966** | **23,746,696** |

**E is zero in each iteration**, and every admitted pair has equal clocks.
Thus **none of S-N in any of the 55 iterations is explained by an admitted
pricing correction**: the entire remaining signed difference is U-O. This is
partial ownership coverage, not an exact iteration pass. Individual equations
are in `target/shadow-m2/m2b-join-final/iterations.jsonl.zst` and its coverage
report.

There are **184 unmatched kernel rows**: 92 `ordered_path_mismatch` rows plus
46 source-only and 46 native-only rows. These include the `$06:EAA4` direction
predicate family: native below/right helper and intervening parent-marker
order differs from source order, notably `$06:EAE8` and `$06:EAA7` in
`sprite/movement_spawn.rs`. No semantic commutation rule is proved, so these
rows are not sorted into apparent agreement. Remaining source/native work is
explicitly unmatched, predominantly scalars and unresolved call/block mappings.

### Worklist and pricing gate

`target/shadow-m2/m2b-join-final/worklist.json` has **zero validated positive
gaps, zero validated overcharge repairs, zero exclusive gap clocks and zero
missing holds covered**. Both validated rankings, by clocks and by holds
covered, are empty. Unmatched native work may still contain overcharges.
Residuals retain ROM PC/end, call path, actual widths, native site candidates,
clocks and affected missing holds, with scalar exclusion explicitly false.
They are investigation rows, not pricing instructions.

**STOP for pricing batches consuming this worklist.** Prove scalar instruction
ownership and resolve the concrete site/ordering residuals with fixtures before
admitting exclusive gaps. The continuous lane's internal-interrupt objection is
resolved; it is not the remaining reason to stop.

### Rate, verification and retained evidence

The source pass takes **799.603 s** including final identity verification:
**75.04 source hosts/s / 944,414 opcodes/s**. Instrumented execution alone is
787.424 s, or 76.20 hosts/s / 959,022 opcodes/s. Concurrent compilation and CPU
checks affected machine load; these are measured prefix rates, not a full-route
projection.

All **26 auditor-package unit tests**, disabled-feature package tests, and
**three filtered native observer tests** pass. Default, no-default-feature and
feature-enabled product builds, and enabled/disabled auditor builds, are
warning-free. Existing lib-test dead-code warnings are separate from ordinary
builds; no full library suite or GPU test ran. The fresh default **200,000-frame
state ledger matches every record and EOF**. On a fresh 35,311-iteration feature
replay, observer off/on predictor, routine, block, object and sprite CSVs and
stdout are byte-identical. `git diff --check` passes.

Evidence is under `target/shadow-m2/m2b-*`: frozen source manifest/code,
source reproduction/rate report, native selection/events/process receipts,
final join call/block/residual streams and coverage/worklist, build/test logs,
observer comparison and verification receipt. Superseded Cargo outputs were
pruned first; disk samples and final pruning keep the workspace below
3,000,000,000 bytes. No GPU, video, cached-av, external Snes9x, protected-cache
or shared-ledger write, pricing/pacing/classifier/P2/marker/table/seeding change,
cargo fmt, commit or push was performed. This remains CPU audit evidence, not
independent hardware or exact A/V acceptance.
