# Lagless audio vs the real original — batch 3

**A full-route verdict about what a player would hear differently is still unfinished.** The real original is now verified for the entire route. The seeded live candidate emitted **636,839 key-on voice edges versus 636,566 original edges (+273 net)**, but that net is not an extra-sound count. Fourteen sound-chip restarts and ambiguous repeated-tone matches prevent turning the provisional alignment into trustworthy music/SFX/dialogue audibility totals. There are **20 difference candidates and five controls from the verified prefix** in `target/lagless-audio-notes3/listen/index.json`; they are not a full-route “20 most audible” ranking.

This is a verified stopping point for A, with B diagnosed and C narrowed. No gameplay, pricing, marker, or RNG implementation changed. The previous report is preserved at `target/lagless-audio-notes3/report-v2.md`; run-2 evidence remains under `target/lagless-audio-pcm/`.

## A. Route-faithful live audio against original S-DSP writes

### Original authority and recorder verification

`--render-original-audio-cpu CACHE CORE ROM FRAMES` runs only the pinned Snes9x 1.63 trace core. It disables video through libretro's audio/video-enable interface, discards video callbacks, creates no GPU resources, and holds `/tmp/zelda3-snes9x-compare.lock` exclusively. It checks the core, ROM, SRAM, oracle ledger and initial serialized-state hashes before replay. It serializes only once before frame zero, matching the cache's initialization; there are no in-run checkpoints, imported RNG answers, or native-reference substitutions.

The full cold original capture matched **all 1,581,079 cached PCM hashes**, **842,907,927 stereo pairs**, and oracle EOF. Its **130,550,798 raw DSP writes** independently rebuild **636,566 KON and 1,018,724 KOFF voice edges**. Core SHA-256 is `6f112667f7b590c3f789630171ce9af79b1793245ad2e14080522146ffd6d498`; oracle-ledger SHA-256 remains `8b849c8ce1040e65016daeccf311768a2b7cfd1017ec7f7f6f48899a1903c6d7`. See `original-full/complete.json` and `full-alignment/summary.json` under the new evidence directory.

The recorder reads the core's existing DSP-register-write buffer, with a fail-closed capacity check. The candidate records `AudioEventFrame.events[].parity_dsp` from its actual live callback. Both retain write order, sample coordinate, phase, host, context, voice, SRCN, pitch, signed stereo volume, ADSR and gain. Independent reconstruction checked every emitted edge in both full streams. The first **1,802** original KON snapshots also match the previously hash-verified default prefix in every register field; timestamp differences are **0 or one sample (0.03125 ms)**. The original has two additional KOFF edges at host 7163, immediately before the known default PCM failure at 7164.

### Candidate route coverage and seeds

The requested lane completed **1,473,123 logical iterations / 1,576,881 callbacks**, with live APU answers and `--seed-every 2000`. Its census gate passed with **zero unexplained differences**. The recorder verified **130,422,750 DSP writes**, **636,839 KON / 1,017,988 KOFF voice edges**. Unlike the cold run-2 candidate, this lane reaches ending modules `$19/$1A`: **23,434 DSP-active callbacks**, versus **23,423** in the original. This is route coverage in the census's seeded sense, not continuous cold play.

Seeds **do touch the APU**. `seed_game` starts a fresh engine, imports memory, loads the resident song bank and restores music from WRAM. The seed contains **no original SPC RAM or DSP continuation**. There are **14 actual SPC restarts**, not a reset at every periodic boundary. Raw capture rows mark them explicitly. Register reconstruction clears at each restart; fields remain unknown until observed writes establish them. One candidate KON lacks an observable source/pitch identity and is excluded from pairing.

The comparison uses **537 segments** cut at matched song commands and actual restarts. Restart intervals remain separately counted until the next matched song request supplies a common music origin. That does not establish identical SFX history after a reset. Notes already active at a cut have censored releases; no new KON is invented at a boundary.

The logical tape's controller words and all **87,590 RNG draws** were independently checked against the read-only oracle inputs. The current main contains pricing work that predates this batch; its census is not byte-identical to the older timing study. Consequently all full-route anchors were regenerated from this run's trace. The prefix listening anchors match the regenerated anchors exactly.

### Provisional note alignment, not audible-event counts

`scripts/lagless_audio_notes.py --segments ...` checks the original PCM proof and both complete raw-DSP/edge streams before comparison. It carries registers across cuts, separates absolute, command-relative and first-matched-note-relative timing, and excludes unobserved register fields from mismatch counts. Times use the common **32 kHz PCM grid**, not the timing report's 60 Hz presentation-frame estimates.

| Alignment result | Outside marked restart intervals | Restart intervals | All |
| --- | ---: | ---: | ---: |
| Exact voice/SRCN/pitch token pairs | 616,305 | 3,842 | 620,147 |
| Unmatched original KON | 11,458 | 1,143 | 12,601 |
| Unmatched candidate KON | 11,842 | 1,031 | 12,873 |
| Substitution pairs | 3,544 | 274 | 3,818 |
| Pitch snapshot mismatches | 3,516 | 274 | 3,790 |
| SRCN snapshot mismatches | 2,858 | 143 | 3,001 |
| Stereo-volume snapshot mismatches | 16,942 | 522 | 17,464 |

| Matched-pair timing threshold | Outside marked restart intervals, >20 / >45 ms | All, >20 / >45 ms |
| --- | ---: | ---: |
| Absolute onset shift | 619,843 / 619,838 | 623,959 / 623,954 |
| Matched-command-relative shift | 211,093 / 133,959 | 214,527 / 136,897 |
| Song-relative first exact note pair | 153,340 / 127,150 | 156,613 / 130,395 |

These are **alignment-convention results**. Repeated tones can pair with a different repetition; SFX can cross a music-command boundary. For example, the first native-only dialogue edge is iteration 3333. Its candidate KON is voice 7 / SRCN 1 / pitch 450 at callback 3704; a same-token original tone occurs at host 3793, before the source song boundary, while the candidate tone falls after its boundary. The sequence matcher pairs that original tone with an earlier candidate repetition. This cannot certify an extra click or its onset shift. The summary therefore leaves every note's category **unattributed**. Snapshot differences also do not validate complete pitch/volume contours, sample-bank identity or audible envelope output.

| Requested category/cohort | Verified finding | Unfinished audio attribution |
| --- | --- | --- |
| Music | Complete original and candidate DSP inventories; matched song anchors | Ownership, repeated-note identity and audible mismatch counts |
| SFX | Raw KON/KOFF and parameter writes retained | Which musical voices were temporarily owned by SFX |
| 636 native-only dialogue command edges | All 636 remain native-only at the same iteration keys in this run | Extra audible clicks versus retrigger/representation differences |
| Older 164 non-`$FF` source-only commands | 152 remain source-only; 12 now match. One new source-only event makes the current total 153 | Which commands actually create missing notes or change an existing sound |
| Song end versus fanfare | Fanfare requests and gameplay zero-poll episodes retained in `live-commands/song-ends.jsonl.gz` | Last musical release/tail separated from SFX and gameplay poll timing |

The remaining A work is to validate per-note ownership and command association through repeated tones and song boundaries, then produce category counts and the genuine full-route top-20 listening selection. The tables above must not be advertised as player-audibility findings.

### Listening evidence

`listen/` contains **50 WAVs / 25 pairs**, exactly **96,000 stereo pairs (3 seconds at 32 kHz)** per file. The index includes id, category, description, context, absolute and relative onset fields, note differences, alignment and hashes. Selection prioritizes unmatched/substituted notes and relative shifts in the first **10,000 original callbacks**; five controls use matched KON snapshots. “Control” does not assert identical mixed PCM.

Each pair uses the same segment's first exact note-pair offset, retaining the candidate's local onset differences rather than aligning each target note independently. The original was recaptured directly from Snes9x and verified against every prefix hash. Candidate DSP and hash streams match the independently verified prefix capture. Every consumed PCM callback was hash-checked before extraction; both raw PCM files were deleted afterward. See `listen-verification.json`. This is a valid scoped listening set, not the requested whole-route audibility ranking.

## B. Where continuous cold lagless leaves the route

A fresh **native-RNG, live-APU, unseeded** logical replay reproduces run 2's first **10,833 PCM callbacks exactly**, covering 10,000 logical iterations. Its trajectory therefore diagnoses the actual run-2 prefix rather than a different audio lane.

| First difference | Logical iteration | Source host | Source → cold native |
| --- | ---: | ---: | --- |
| WRAM / RNG seed `$0FA1` | **4363** | 4975 | `$45` → `$DD` |
| Link X | **8613** | 9544 | 2742 → 2741 |
| Link Y | **8651** | 9583 | 2892 → 2893 |
| Module/submodule/subsubmodule | **9336** | 10272 | `$07/$0A/$00` → `$0E/$02/$00` |

There is no room-field difference in this 10,000-iteration prefix. Controller words are unchanged. At 4363 the original supplies RNG results `$22`, then `$45`; native ends at `$DD`. `get_random_number_with_carry` explicitly uses a deterministic fallback derived from the seed and frame counter when there is no source RNG replay. That fallback does not implement the original's PPU-read RNG routine. The requested source-RNG census is WRAM-clean throughout the same prefix; removing only its RNG replay produces this first seed difference.

**Bug class: divergent RNG semantics / missing source entropy behavior.** The evidence does not require an input-tape phase error or a newly unpriced lag. Frame-counter dependence is part of the native fallback, but restoring timing alone would not make that fallback implement the original PPU reads. The exact collision/branch causing the first pixel of positional divergence has not been instruction-traced. Evidence: `cold.log`, `cold-outcomes.jsonl.zst`, `cold-boundaries.json`, `cold-run2-equivalence.json`, and `trajectory-and-producer-findings.json`.

## C. Iteration 6437: `$31` / `$0B`

The source-RNG live lane **does send both commands**: native logical iteration **6436**, global callback **7047** (engine debug host **7048**), versus source iteration **6437**, source host interval **7162–7163**. This is the census's expected main/NMI publication offset, not a missing lagless command.

The existing producer diagnostics identify **sprite slot 13, type `$E3` (fairy)** and `sprite_handle_absorption_by_player`:

- The absorption SFX table supplies port-3 `$0B` (`sprite/lifecycle.rs`, diagnostic caller line 2058).
- Type `$E3` gives case 11, which queues port-2 `$31` and adds 56 to the heart filler.
- The source-RNG replay and source image both contain the fairy; the native-RNG cold replay has an empty slot instead and reports both commands missing at 6437. Thus the cold lagless omission is a pickup/outcome difference before the audio queue, not evidence of a late SPC rendering of that pickup.

Default mode remains **absent by its host-7164 failure**, as run 2 established. This batch verifies the responsible native producer and the native-RNG pickup divergence, but does not capture default-mode future publications to rule out every later equal-valued command. It must not be described as a proven default queue-delay defect. Evidence: `producer.log`, `producer-wram/`, `command-6437-live.json`, and the retained run-2 boundary journal.

## Checks, artifacts and reproduction

All new capture/evidence output is under `target/lagless-audio-notes3/`. `verification.json` binds final artifacts and source hashes. The ordinary default and feature-off builds are warning-free; **27 Python audio tests** and the existing **1,000-callback native/product resampling test** pass. The feature-off binary rejects the new CPU capture mode before frontend startup. The existing default **200,000 records and EOF match unchanged**. Logical/state ledgers were read only.

The first full original capture's `music.last_music_control` metadata contains the `$0130` **request latch**; it is not used for song identity or the alignment. Current capture code correctly labels `$0130` and `$0133` separately. Raw DSP writes, edge reconstruction and PCM hash proof are unaffected; song identities come from the freshly generated command trace.

Build with `ZELDA3_ASSETS_DIR=/Users/missingno/Documents/zelda3-rs/generated/zelda3_assets cargo build --offline --profile parity -p zelda3-bin`. Use isolated save directories and the JSON plans retained in this evidence directory. The CPU-only capture modes are:

```sh
# ZELDA3_DEBUG_PCM_WINDOWS points to an output/ranges JSON plan.
# CACHE is the read-only ed0121e1... oracle cache; CORE is the pinned trace dylib.
target/parity/zelda3 --render-original-audio-cpu "$CACHE" "$CORE" "$ROM" 1581079

target/parity/zelda3 --logical-replay "$LEDGER" "$CACHE/initial.srm" \
  --seed-every 2000 --live-apu-reads --report 0 \
  --audio-timing-trace "$OUTPUT/live-full-timing.jsonl.zst"

python3 scripts/lagless_audio_notes.py "$OUTPUT/original-full" "$OUTPUT/live-full" \
  "$OUTPUT/full-alignment" --segments "$OUTPUT/segment-boundaries.json" \
  --oracle "$CACHE/oracle-av-hashes.jsonl"
```

Existing capture files are not overwritten. Disk was checked before long runs; this study remains under **3 GB**. No GPU run, live video comparison, cached-av execution, oracle-cache write, ledger modification, `cargo fmt`, commit or push was performed.

## Live sound continuous 1

**Verified diagnosis stop at main `f4e5641b`, branch `codex/live-songend1`; no runtime fix.** Cold replay uses recorded inputs/RNG, `--live-apu-reads`, and no `--seed-every`. It reproduces the first live divergence at **94,848**. The clean prefix contains 94,848 iterations (last clean index **94,847**); the diagnostic run completes 94,857 iterations and its required census gate **FAILS with nine song-state comparisons**. No later live frontier is claimed. Evidence is under `target/live-songend1/`, principally `diagnosis.json`, `recorder-verification.json`, `native-timing.jsonl.zst`, `native.log`, and `original/ports.jsonl`.

| Live divergence | Mechanism | Fix | Furthest continuous live result |
| --- | --- | --- | --- |
| **94,848**, `$07/$00`, room `$00C8`; native song poll zero versus original `$13` | **Class 5, premature publication during held item-receipt work**, plus different callback-clock and poll phases. Native has given the song 100,782 more SPC cycles when this poll reads its acknowledgement. | None. A general publication/observation phase repair remains required; no fitted delay, forced answer, or song exception was added. | **94,847 clean**; diagnostic execution through **94,856**. |

### Absolute SPC measurements

The pinned renderless Snes9x capture matches **103,741 consecutive original PCM hashes**, covering **55,306,534 stereo pairs**. Independent reconstruction verifies every emitted edge against raw DSP writes: original **8,566,550 writes**, native counts in `recorder-verification.json`. Both lanes start cold; no SPC checkpoint initializes either capture. Sparse CPU APUI and SMP output-port receipts were added to the existing original-audio capture harness, sharing its selected PCM ranges and fail-closed trace-capacity checks. Gameplay, pricing and audio runtime source are unchanged.

All coordinates below are **absolute SPC cycles**, not callback counts or milliseconds. Original CPU receipt cycles are reconstructed from the cumulative output sample count and the core's `apu_cycle_after`. The DSP sample counter increments after phase 27: subtract 32 at raw phases **28–31**. This mapping independently reproduces **all 4,217** absolute SMP output-store timestamps in the window, including phases 27, 28 and 29. Native note timestamps are `sample * 32 + phase`; their SPC rebase origin is retained separately in the instruction trace.

| Event | Original absolute SPC cycle | Native absolute SPC cycle | Original since command | Native since command |
| --- | ---: | ---: | ---: | ---: |
| `$13` CPU-to-SPC port write | 1,758,669,007 | 1,757,415,403 | 0 | 0 |
| Driver output port 0 becomes zero | 1,769,555,263 | 1,768,298,319 | 10,886,256 | 10,882,916 |
| Final all-voice KOFF | 1,769,555,344 | 1,768,298,400 | 10,886,337 | 10,882,997 |
| Poll at logical iteration 94,848 | 1,769,453,260 | 1,768,300,438 | 10,784,253 | 10,885,035 |

The native poll reads the acknowledged state from the preceding audio callback: its output cursor is **1,768,300,416**, with actual SPC execution **22 cycles ahead**, at 1,768,300,438. It does not advance to a timed CPU bus read in lagless mode. The original reads `$2140` at `$08:C400`, host **103720**, **V0/H218**; its receipt places the chip at 1,769,453,260. The native zero-store cycle comes from `state-poll/0-spc.txt`: `$08E5 MOV $00F4+X,A`, `X=A=0`, starts at 1,768,298,313 and stores on the final cycle of its `[4,1,1]` sequence. Both chips emit the final KOFF exactly **81 cycles after the zero store**.

### Which term differs

The apparent early song end is dominated by **early command arrival relative to the logical CPU work**, rather than a song shortened by six callbacks. Iteration **94,215** consumes **five callbacks / four holds** on the native side. Its first callback, **102845**, already delivers `$13`; all five native callback queues contain the new song. Original callbacks **103083–103087** still deliver the old command state. A separate cold, PCM-verified `wram,nmi` trace shows the request latch `$012C=$13` authored only at **host 103087, V235/H76, `$09:88B7`**, after those held NMIs. The actual original port store follows at **host 103088, V225/H530, `$00:80F6`**, in iteration **94,216**. The earlier observation of 633 callbacks and zero holds from 94,216 through 94,848 therefore began **after** the premature native publication.

The native producer is `misc.rs::complete_ancilla_add_item_receipt`, after graphics work in `ancilla_add_item_receipt_from`; `rtl_native_pacing.rs` publishes the completed semantic main state before playing its presentation holds. Its sound checkpoint handles selected SFX queues, not a general timed music-command publication. A repair must preserve the source producer phase across graphics/caller suspension and NMI, including one-shot delivery on resume; simply moving every song to the last callback has not been justified.

Normalize both clocks at the start of logical iteration 94,216, then compare command-to-poll intervals:

| Contribution to native extra song time at poll | SPC cycles |
| --- | ---: |
| Command publication phase: native −85,397 versus original +47 from that boundary | **+85,444** |
| Callback clock over the same logical span: native 10,799,616 versus original 10,781,856 | **+17,760** |
| Poll observation phase: native +22 versus original +2,444 | **−2,422** |
| **Total**, independently equal to 10,885,035 − 10,784,253 | **+100,782** |

The native lane advances a fixed **534 pairs per callback**; the original emits its actual variable sample counts. This clock contribution is distinct from the SPC timer/driver tempo. The song's command-to-final-KOFF interval differs by only **−3,340 cycles**, below two 2,048-cycle driver timer periods. That residual initial/timer/instruction phase has not been fully attributed and is not claimed exact.

Both captures contain **221 KON voice edges** over the song window, with identical **per-voice ordered SRCN and pitch sequences**: counts by voice are **31, 18, 31, 31, 31, 31, 23, 25**. Cross-voice ordering sometimes differs; `per-voice-onsets.json` retains the comparison. There is no extra/missing KON in these per-voice sequences and no evidence of a sustained tempo-rate error large enough to explain the boundary. This does not assert identical mixed PCM, note envelopes, pitch contours, or sample RAM. The source poll is **102,003 cycles before** its zero store; the native poll is **2,119 cycles after** its zero store.

### Verification and limits

The ordinary capture build is **warning-free**, and `git diff --check` passes. The asset-backed audio library selection passes **192 tests**. Its first invocation ran before the required root asset link existed and failed eight missing-file fixtures; the corrected run is `audio-tests-assets.log`. The library test build also reports existing unused-method warnings; those were not changed or suppressed. The temporary asset link was removed after testing.

No runtime change was made, so the full seeded censuses and default 200k ledger comparison were **not rerun**. The supplied main ROM census **3,838 missing / 0 excess / 0 false positives** and the previously verified default 200k equality remain prior evidence, not fresh checks of a repair. Fresh continuous live-prefix pacing is **8,395 held versus 8,542 source lag NMIs**, with 250 masked callbacks; this is neither a seeded full-route live count nor a passing continuous gate. There are no before/after repair counts or boundary-regression claims because no repair was installed.

`verification.json` binds the capture binary, source, plans, receipts and derived evidence. The original and producer captures honor the compare lock and disable video. Artifacts remain below **3 GB**. No GPU run, ledger write, oracle-cache write, cargo fmt, commit or push occurred. The next implementation must resolve general publication and live-poll ownership, test before/equality/after its timing boundary, and then run both seeded gates, the default 200k check and a fresh continuous live replay.

## Live sound continuous 2

Two general lagless mechanisms are implemented in the uncommitted working tree based on main `0f08d0f1`. Request publication now records each write to `$012C..$012F` at its cycle-ledger prefix, uses the existing P2 elapsed-clock rule (including refresh, interrupt/worker ownership and raster waits), and applies the write exactly once before the corresponding callback's NMI samples it. Equality advances to the next callback slot. Music, ambient, both SFX bytes, word setters, clears, and compatibility WRAM writes use the same path. Multiple writes within a slot retain last-write-wins behavior; a deadline beyond an iteration survives into the next one. No song ID, source lag count, oracle answer, or added callback delay selects a deadline. The existing SFX-consumer checkpoint clears only a request whose priced NMI has elapsed.

### Continuous live frontier

The final CPU-only replay uses recorded inputs/RNG, `--report 0 --live-apu-reads`, and no `--seed-every`. It completes 94,860 iterations, with **94,853 clean iterations, indices 0–94,852**. Ordinary WRAM, RNG, APU-read consumption and ordinary command comparisons remain clean. Its strict summary gate still **FAILS with seven song-state differences** after the next live divergence; this is not a full continuous pass.

| Iteration | Mechanism and evidence | Status |
| --- | --- | --- |
| **94,848** | Premature command publication plus fixed-534 callback clock from continuous 1. The poll now reads live nonzero, matching source `$13`. | Fixed at this boundary; uninterrupted replay passes it. |
| **94,853** | Remaining producer-prefix placement and poll-phase gap. Native reads zero; source still reads `$13`, then reads zero at 94,854. P2 places `$13` on callback **index 3 of five** in iteration 94,215, rather than the first callback. The original request was authored after its four held NMIs, with the port store in 94,216. Prefix pricing/observation is still insufficient for exact live timing. | Not fixed; no extra delay or forced poll answer was added. |

`frontier-timing.json` retains the selected raw observations. Iteration 94,215 starts at native callback 102,844; its queues retain `$15` for callbacks 102,844–102,846 and first contain `$13` at **102,847**, then retain that command state at 102,848. Iteration 94,216 starts at 102,849. Repeated queue values are retained command state, not repeated producer stores. Native polls remain nonzero through 94,852. Evidence: `continuous.log`, `continuous-gate.log`, `continuous-timing.jsonl.zst`, `state-command/`, and `state-poll/`, all under `target/live-songend2/`.

### Fractional callback clock and playback

`LaglessSampleClock` accumulates the existing Snes9x ratio **15,664 / 328,125 SPC cycles per master clock**, divided by **32 SPC cycles per stereo pair**. It uses the existing non-interlace field clock: 357,368 master clocks, with the four-clock short scanline every other field. No callback rounds its duration independently. The long-term budget is **533.122002285714 stereo pairs per callback**, or **28.095927 fewer SPC cycles per callback** than the former 17,088-cycle budget. Upload-return forecasting peeks the same next window, without advancing the accumulator. Default/ROM-timed advancement retains its previous path.

The hardware accumulator is **not an exact per-host oracle count reconstruction**. The retained PCM-verified capture contains 103,741 hosts: one 457-pair reset partial frame, 90,419 hosts with 533 pairs, 12,989 with 534, and **332 with 532**. Across hosts 1–103,740, the nominal field accumulator produces **55,306,076 pairs**, versus the oracle's **55,306,077**. Its cumulative difference stays between −2 and 0 pairs in that measured window; **95,623 / 103,740 individual host counts match**. Instruction-boundary callback jitter and the reset partial frame are not supplied by a field-duration accumulator. Thus the requested exact per-host proof remains unmet; no oracle counts were fed back into runtime scheduling. `clock-oracle-verification.json` records the source hash and complete error distributions. For the diagnosis’s specific **632-callback interval from 94,216 to 94,848**, the final native clock advances **336,933 pairs**, exactly the source interval’s count, instead of 337,488: the former **17,760-cycle clock excess is removed**.

Stereo playback renders the actual 533/534-pair native window before resampling to the requested output size, including **735 pairs at 44.1 kHz**. Linear interpolation follows the continuous fractional clock and carries two previous pairs across callbacks, avoiding block padding, gaps and phase resets. Its causal history adds two native pairs of latency. At an unchanged callback rate, playback is **0.164419% slower**, pitch is **2.848821 cents lower**, and a passage takes **0.164690% longer** than the former fixed-534 path. This is a rate calculation and CPU output-continuity test, not an audio-device audition. Version-9 audio snapshots retain the fractional clock and interpolation history, preserve version-8 decoding, and the outer play checkpoint magic is bumped to `Z3RSPC27`.

### Verification

The final production build is warning-free. The **194 audio tests**, **16 pacing tests**, and the exact **two-million-callback accumulator test** pass. Boundary tests cover all four request latches before, at and after a hold boundary and verify one-shot consumption. Playback tests cross alternating native window sizes with a continuous ramp, checking stereo agreement and no boundary gap; snapshot tests verify the clock/history and subsequent output after restore. The library-test build reports **24 pre-existing unused-method warnings**; these were not suppressed.

The final binary's default **200,000-frame ledger comparison passes every record and EOF**, including state, display, SRAM and audio hashes. The source ledger and oracle cache were read-only. No GPU run, `cargo fmt`, commit or push was performed.

### Full seeded censuses

Both final lanes complete **1,473,123 iterations** with **`PASS (unexplained=0, complete=True)`**. Each uses `--seed-every 2000` and recorded inputs/RNG. These gates permit the existing classified song/lag/polygon differences and do not supersede the continuous frontier above.

| Measure | Before live, fresh retained baseline | After live | After ROM answers |
| --- | ---: | ---: | ---: |
| Missing holds | 4,535 | **4,477** | **3,838** |
| Excess holds | 590 | **520** | **0** |
| Zero-source-lag iterations with a native hold (FP) | 132 | **98** | **0** |
| Held callbacks | 98,732 | 98,720 | 98,839 |
| Masked callbacks | 5,119 | 5,126 | 5,145 |

The ROM lane exactly retains the supplied main baseline of **3,838 missing / 0 excess / 0 FP**. Live missing holds fall by **58**, excess by **70**, and FP by **34**. The fresh measured before totals are 590 / 132 rather than the brief's approximate 596 / 133; the table uses the actual retained-binary capture. All held-frame decisions in the common clean prefix through iteration 94,847 are unchanged (`clean-prefix-hold-changes.json`).

Evidence: `before-live-gate.log`, `after-{rom,live}-gate.log`, corresponding replay logs and predictor CSVs, and `census-totals.json`. The before binary was retained before building the repairs. The after gates, final continuous capture and final default-200k comparison use `final-zelda3`.

The complete default library suite finishes with **2,693 passed / 1 known failure / 3 default-ignored tests**. The sole failure is `cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`, with the existing `Some(1047868)` versus `Some(170442)` assertion. `library-tests-final.log` retains the full result. No additional failure was hidden by the audio or pacing selections.

The retained artifact set is about **0.65 GB** after removing build caches, and the temporary asset-input symlink is removed. Build intermediates briefly exceeded the requested 3 GB cap during compilation; they were pruned rather than retained. `verification.json` binds the final and baseline binaries, test executable, source patch, inputs, gates and reported limitations. All retained output is under `target/live-songend2/`.
