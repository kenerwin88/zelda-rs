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
