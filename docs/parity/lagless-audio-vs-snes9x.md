# Lagless audio versus pinned Snes9x

Measurement branch: `codex/lagless-audio-compare`, starting at main
`8dbe21027bbb4b5f234c4499df6ec218e8aa56d5`.

The lagless soundtrack is **not sample- or timing-equivalent** to the pinned
Snes9x stream. With cartridge APU answers, the continuous route emits
2,810,730 fewer native stereo pairs (87.835 nominal seconds), and only 660
mapped iterations are sample-exact. Uninterrupted live SPC changes gameplay
following an earlier song-end answer; it executes the complete input stream
but does not reproduce the source quest trajectory. Its much larger duration
deficit must not be called a same-quest pacing improvement. Aligned waveform
residuals remain. Live renderless traces now show unequal port delivery and
DSP-register input in both investigated windows; no equal-input SPC/DSP
emulation defect has been isolated.

## Scope and authority

The authority is cache
`ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18`
under `/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/`.
`original-timing-host-receipts.jsonl.zst` contains the actual interleaved
stereo i16 samples; `oracle-av-hashes.jsonl` independently binds each host's
PCM bytes, channel count and sample count. Extraction checks every frame
against those hashes. No source PCM is synthesized from a Rust replay.

The lagless driver is the supplied logical ledger at
`/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full`.
Those receipts retain their original ledger provenance. New runs use the
corrected `full-v2`; `full` remains the immutable backup. See
[Continuous lagless 2](cost-pacing.md#continuous-lagless-2).
It runs the existing production pacing holds and audio calls. Cartridge APU
answers and live SPC are separate executions; both retain the ledger's
cartridge RNG policy. This measures that requested driver configuration,
not a recording made with the shipped game's native RNG. No oracle receipts
are installed into the lagless audio runtime.

Two executions per APU policy distinguish continuous cold execution (no
`--seed-every`) from the existing `--seed-every 2000` census. A census can
replace gameplay state and reconstruct its audio owner; its reseeds are
explicit in the mapping and must not masquerade as continuous playback.
Executing every recorded input is also distinct from showing that a drifted
replay reaches the cartridge's gameplay outcomes.

## Method

The existing `--audio-pcm FILE` writes exactly what `zelda_render_audio`
returns, including held and masked callbacks. The new `--audio-map FILE`
writes one JSONL row per completed iteration: source opening host and next
opening host, native sample range, callback/held/masked counts, module,
submodule, room, current music byte, poll observations and actual reseeds.
Mapping is an observer; it neither supplies holds nor changes an audio budget.
`--audio-only` additionally skips the WRAM/command census for continuous cold
replays. It requires PCM and mapping outputs, forbids seeds/video/start/offset
and omitted census outputs, and retains pacing-predictor CSV support. The
normal census path remains available for source/gameplay context checks.
Each normal callback emits 534 native stereo pairs. Sample coordinates count
stereo pairs, not individual i16 channel values. Milliseconds use the nominal
32,000 Hz DSP rate; one comparison frame is 534 pairs (16.6875 ms).

Source spans are `[open_host(k), open_host(k+1))`, in zero-based host calls.
They are whole host audio buffers, not sample-accurate cuts at the NMI:
two opening NMIs may share a host, leaving a zero-host span, or straddle
host calls. The first ledger iteration opens at host 81, so the oracle's
bootstrap prefix is excluded from the route-relative duration origin and
reported separately. At the last iteration, the source span extends to the
oracle EOF because no next opening exists. That terminal convention is counted.

Exact iteration comparison requires identical length and every stereo value.
Exact frame comparison pairs native callback j with source host open_host+j;
the denominator is max(native callbacks, source hosts) for each iteration.
Unpaired callbacks or hosts remain mismatches. These are explicit mapping
statistics, not proof of instruction-boundary audio equivalence.

For time alignment, sample a 2,048-pair (64 ms) window every 60 iterations
and additionally at each change in current music byte. Stereo FFT correlation
searches every integer shift within ±3,200 pairs (100 ms) of two candidate
positions: the source-mapped position, and that position adjusted by the
independent sample-count drift accumulated since the current music byte changed
or a reseed occurred. This second center permits long songs to accumulate
drift beyond 100 ms without discarding all correspondence windows. The best
correlation of the two regions wins; distant competing matches are ambiguous.
This phase-center prediction is a search hypothesis, not evidence that a song
actually reset its audible phase. Both channel energies are normalized after
removing each channel mean.
A usable match requires correlation ≥0.95, no search-boundary winner and no
equal-best plateau or competing peak more than 534 samples away within 0.001
of the winner. Silence, weak matches, ambiguous peaks and boundary matches
are unresolved, not measured onsets. These are waveform correspondence
windows; they are not independently labelled DSP key-on events. There is
no claim that the best shift inside this finite radius is a global match.

For each usable correspondence, report both the local source-position offset
and the continuous-stream offset, in samples and nominal milliseconds.
The continuous offset is local offset plus the cumulative sample-count
difference *before* the window; positive means native occurs later.
Bins are disjoint: exact zero, nonzero within 534, 535–1,068, and greater
than 1,068 pairs. Full-window byte equality is checked independently of
correlation; a strong correlation is never sample-exact proof.

Duration drift is native emitted pairs minus source pairs since host 81.
Negative means less native audio elapsed at the corresponding route marker;
positive means more. The accumulator is never reset at a music change or
reseed. Music changes and periodic checkpoints retain its value, allowing
reset claims to be checked rather than inferred from the end total.
The current music byte is game context, not an independently detected audible
song onset.

## Reproduction and artifacts

All measurements and logs are under `target/lagless-audio-proof/`. The
`compare-zelda3` binary is the default-feature full-route capture build;
`final-zelda3` adds the optional audio-only shortcut and is the final default
binary. `final-reduced-zelda3` is built with `--no-default-features --features parity-debug`.
The independent main baseline is built from a `git archive main` tree in
`/tmp/zelda3-audio-main`, with the same source asset directory and ROM path.
Only `clean-main-zelda3` counts as the clean-main baseline; its source archive
was independent of ongoing instrumentation edits. The capture and final
binary hashes are retained separately in `verification.json`.
Both final audio-only policies reproduce the first 20,000 captured PCM windows
and mapping rows byte for byte, and produce all 20,000 pacing CSV rows. See
`audio-only-prefix-proof.json`. The full-route measurements use the original
normal-census capture path; no shortcut run is substituted for them.

```sh
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
L=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
P=target/lagless-audio-proof
python3 scripts/lagless_audio_compare.py extract "$C" "$P/oracle"
ZELDA3_SAVE_DIR=/tmp/lagless-audio-rom "$P/compare-zelda3" \
  --logical-replay "$L" "$C/initial.srm" --seed-every 2000 --report 0 \
  --pacing-predictor "$P/rom.csv" --audio-pcm "$P/rom.pcm" \
  --audio-map "$P/rom.jsonl" > "$P/rom.log" 2>&1
# Use separate paths and --live-apu-reads for live SPC.
# Omit --seed-every for continuous-rom / continuous-live.
python3 scripts/lagless_audio_compare.py compare "$P/oracle" \
  "$P/rom.pcm" "$P/rom.jsonl" "$P/rom-summary.json"
python3 scripts/lagless_audio_compare.py modes \
  "$P/rom.pcm" "$P/rom.jsonl" "$P/live.pcm" "$P/live.jsonl" "$P/modes.json"
python3 -m unittest discover -s scripts -p test_lagless_audio_compare.py
```

Extraction requires NumPy and the `zstd` CLI. Outputs are exclusive-create to
avoid silently replacing an extracted oracle stream. Summary JSON retains
all drift checkpoints, music changes and worst windows; the adjacent
`.windows.jsonl` retains every alignment attempt and its confidence fields.

`--audio-state-at ITERATION:DIR` saves full before/after native audio snapshots
and SPC instruction trace/clock/voice state for each callback of that iteration.
`first-sample-state/` retains iteration 2 and `first-correlated-state/` retains
iteration 200. A four-iteration trace-on probe has byte-identical PCM to the
same trace-off prefix.

## Results

All oracle samples were verified: 1,581,079 hosts and 842,907,927 stereo
pairs. The logical route covers 1,580,998 of those hosts and 842,864,820 pairs;
the excluded 81-host bootstrap contains 43,107 pairs. The terminal iteration
uses the explicitly counted EOF convention.

### Complete cartridge-answer and seeded live-SPC censuses

Both completed all **1,473,123 iterations**, without a reported panic.
Cartridge answers required **zero actual reseeds**; its audio history is
continuous despite the census being allowed to reseed. Seeded live SPC
performed **14 actual reseeds** and must not be presented as uninterrupted
shipped-game audio. The separate uninterrupted live result follows below.
A second full cartridge-answer run with no seed option produced byte-identical
PCM and mapping: PCM SHA-256
`fc44441803fe3b56495a31d37f237577e02fc3eb243d7c3ed81098a049837323`,
mapping SHA-256 `9de5593e178d27fde36a75977fdddcd65948f78a2589336113ce9abee44ce67c`.
See `continuous-rom-proof.json`.

| Measure | Cartridge answers | Live SPC, seeded census |
|---|---:|---:|
| Native audio callbacks | 1,573,135 | 1,572,992 |
| Accepted held callbacks / source lag NMIs | 97,288 / 102,677 | 97,181 / 102,677 |
| Native masked callbacks | 2,724 | 2,688 |
| Sample-exact iterations | 660 | 656 |
| Of those, containing nonzero samples | 613 | 608 |
| Exact callback/source-host pairings | 1,956 / 1,948,319 | 1,885 / 1,948,900 |
| Of those, containing nonzero samples | 1,400 | 1,342 |
| Native emitted stereo pairs | 840,054,090 | 839,977,728 |
| Final route-relative sample deficit | −2,810,730 | −2,887,092 |
| Final duration deficit, nominal seconds | −87.8353125 | −90.221625 |

The pairing denominators include unmatched slots in each iteration, as defined
above; they are not a count of unique physical oracle hosts. All exact checks
include length. Most of the soundtrack is not sample-exact on this mapping.
First length mismatch: iteration 0. First overlapping sample mismatch:
iteration 2, source host 84, pair 361.

### Time-aligned correspondence windows

These distributions describe the finite-search, confidence-filtered waveform
windows, not a census of independently identified sound-event onsets.
Bins are disjoint and all numbers below include only usable matches.

| Local source-position offset | Cartridge answers | Live SPC, seeded census |
|---|---:|---:|
| Exactly zero | 0 | 0 |
| Nonzero within one native frame (534 pairs) | 272 | 266 |
| Greater than one, within two frames (1,068 pairs) | 539 | 536 |
| Greater than two frames | 3,046 | 2,909 |
| Usable matches | 3,857 | 3,711 |
| All windows attempted | 25,384 | 25,383 |
| Unresolved windows | 21,527 | 21,672 |
| Sample-identical after alignment, usable matches | 22 | 19 |
| Identical at a best shift including ambiguous matches | 27 | 32 |

The source-position offset median is −1,530 pairs (−47.8125 ms) with
cartridge answers, versus −1,468 (−45.875 ms) with seeded live SPC. Cartridge
5th/95th percentiles are −9,079.8 / +23,321.2 pairs (−283.744 / +728.787 ms);
live values are −8,131.5 / +18,458 (−254.109 / +576.813 ms). Search regions can
be farther than 100 ms from the source position because the independent
sample-budget phase prediction recenters the second region.

On the **continuous playback clock**, every usable match is more than two
frames earlier in native playback: exact/within-one/within-two counts are
0/0/0, and worse counts are 3,857 and 3,711 respectively. Cartridge median is
−1,326,451 pairs (−41,451.594 ms), range −2,823,333 to −4,709 pairs
(−88,229.156 to −147.156 ms). Seeded live median is −1,368,349
(−42,760.906 ms), range −2,899,752 to −4,709 (−90,617.25 to −147.156 ms).
These are observed correspondences; the 85% unresolved coverage prevents
extrapolating those bins to every sound or every sample in the route.

### Duration drift and music changes

An exact accounting identity separates duration effects:

`native − source pairs = 534 × (native callbacks − source hosts) + (534 × source hosts − source pairs)`.

For cartridge answers, the callback-count component is **−4,198,842 pairs**
(−131.2138125 s), partly offset by the fixed-native versus variable-source
sample-budget component **+1,388,112 pairs** (+43.3785 s). Their sum is
−2,810,730 pairs. Seeded live has −4,275,204 + 1,388,112 = −2,887,092 pairs.
Thus the net deficit must not be attributed solely to missing accepted holds:
source host spans also contain elapsed windows without a wait-loop opening,
including masked bank transfers. There are 7,863 fewer native callbacks than
source hosts with cartridge answers, while the accepted-hold deficit is 5,389.
The remaining 2,474 windows are a distinct elapsed-host/transport accounting
difference, not evidence of wrong DSP sample arithmetic.

Cartridge duration drift reaches −2,825,908 pairs before ending at −2,810,730;
seeded live reaches −2,902,270 before ending at −2,887,092. It is not strictly
monotone because of the positive fixed-sample-budget contribution. The global
count is retained at all 843 observed music-byte changes in each run; changing
a song does not restore omitted elapsed audio duration. Song commands can
re-anchor local musical phase, but the finite-window matches do not establish
that every song or sound has relocked. Raw music-byte changes include control
commands and are not labelled audible song starts.

### Uninterrupted live SPC: complete inputs, diverged gameplay

The no-seed live-SPC run completed all 1,473,123 iterations without a reported
panic, with **zero reseeds**. It ends in native module `0E/02`, room `$104`,
music `$03`, while the mapped source entry is `1A/22`. It therefore measures
the uninterrupted lagless response to the recorded input sequence, not the
same completed quest. The first boolean reply difference remains iteration
94,849; the first cross-policy PCM difference remains 94,850. Later policy
changes affect gameplay work, holds, RNG consumption and commands as well as
the soundtrack.

| Measure | Uninterrupted live SPC |
|---|---:|
| Audio callbacks / accepted holds / masked callbacks | 1,528,231 / 53,586 / 1,522 |
| Sample-exact iterations / nonzero exact iterations | 321 / 284 |
| Exact frame pairings / pairing denominator | 1,026 / 1,950,778 |
| Nonzero exact frame pairings | 695 |
| Emitted native stereo pairs | 816,075,354 |
| Final route-relative pair difference | −26,789,466 |
| Final nominal duration difference | −837.1708125 s |
| Callback-count component + fixed-budget component | −28,177,578 + 1,388,112 pairs |
| Most negative cumulative difference | −26,817,995 pairs |
| Observed music-byte changes | 506 |
| Alignment windows / usable / unresolved | 25,051 / 1,692 / 23,359 |
| Local exact / within one / within two / worse | 0 / 142 / 288 / 1,262 |
| Continuous-playback exact / within one / within two / worse | 0 / 0 / 0 / 1,692 |
| Sample-identical after alignment, usable matches | 11 |
| Identical at a best shift including ambiguous matches | 15 |

Local-offset median is −1,459 pairs (−45.59375 ms); 5th/95th percentiles
−4,298.8 / +2,369.35 pairs (−134.3375 / +74.0422 ms). Playback-clock median
is −567,297 pairs (−17,728.03125 ms), with observed range −16,883,282 to
−4,709 pairs (−527,602.5625 to −147.15625 ms). The usable matches cover only
6.8% of attempted windows, and after gameplay divergence a correlated common
menu effect need not be the same source event. These values are waveform
correspondences, not a validated full-route onset-latency distribution.

Its worst retained timing match is iteration **1,140,600**, native `0E/07`,
room `$104`, music `$03`: local offset −742 pairs (−23.1875 ms), playback
−16,883,282 pairs (−527,602.5625 ms), correlation 0.96476, aligned RMS error
1,065.15. Classification: live-reply/control-flow history plus accumulated
pacing/transport differences; the residual cannot be assigned uniquely to
SPC/DSP emulation.

No duration accumulator reset occurs at music changes. The continuous live
track byte stops changing after iteration 957,414, while the supplied source
inputs continue through the ending. The large final count difference reflects
this changed execution. It is not a demonstrated audio-emulator clock drift
under equivalent command input.

All three distinct histories are plotted below. Global counters retain history
across song changes; the seeded/live curve does not erase duration deficits
when a reconstructed audio owner is installed.

![Full-route cumulative audio sample-count drift](../../target/lagless-audio-proof/drift.png)

[Standalone SVG plot](../../target/lagless-audio-proof/drift.svg).

Including the unmatched oracle bootstrap rather than choosing host 81 as the
origin changes the final deficits to **−2,853,837**, **−2,930,199** and
**−26,832,573** pairs for cartridge, seeded live and continuous live respectively.
The route-relative numbers above deliberately report the 43,107-pair origin
difference separately.

### Worst retained windows

Context below is the native entry module/submodule and WRAM room identifier;
room `$A0` can retain the last interior room while outdoors. These rows are
usable correlations, not the unresolved windows with the lowest scores.

| Run / iteration | Context | Local offset | Playback offset | Correlation / aligned RMS error | Interpretation |
|---|---|---:|---:|---|---|
| Cartridge / 1,445,220 | `07/00`, room `$000`, music `$1F` | −882 pairs / −27.563 ms | −2,823,333 / −88,229.156 ms | 0.95836 / 449.26 | Accumulated pacing/transport duration difference; waveform residual unresolved |
| Seeded live / 1,444,620 | `0E/02`, room `$000`, music `$1E` | −1,420 / −44.375 ms | −2,899,752 / −90,617.25 ms | 0.97921 / 592.23 | Pacing plus live-policy/reseed history; residual unresolved |
| Both / 134,700 | `07/00`, room `$043`, music `$11` | −500 / −15.625 ms | −229,562 / −7,173.813 ms | 0.95862 / 3,477.21 | Worst retained correlated amplitude residual; shared by cartridge-answer and live modes; not unique to live replies; DSP defect not isolated |

The summary JSON retains twenty worst timing windows, twenty worst correlated
residual windows, twenty weakest correlations, every drift checkpoint and all
music-byte changes. The adjacent window JSONL retains all attempted windows.


## Cause boundaries and first instances

- **Pacing:** iteration 0, module `00/00`, native held count 0 versus source
  lag-NMI count 1. Native emits 534 pairs against the mapped source span's
  1,066 pairs. This is already a duration mismatch while both overlapping
  buffers are silent. The first actual overlapping sample difference is
  iteration 2, source host 84, pair 361: native `[0, 0]`, oracle `[-18, -18]`.
  Neither observation alone diagnoses a DSP arithmetic defect.
- **Live-SPC answer/control flow:** at iteration 94,849, module `07/00`, room
  `$C8`, cartridge poll answer `$13` is nonzero while live SPC returns zero.
  The first native PCM difference between those policies is iteration 94,850:
  cartridge execution has one callback in module `07/00`; live execution enters
  `0E/02` and takes six callbacks (five holds). The output sample origins were
  identical immediately before this difference. `modes.json` retains the
  first policy differences and their contexts; no oracle mismatch is needed
  to establish this causal policy effect.
- **SPC/DSP emulation:** no first *proven emulation defect* has been isolated.
  A time shift cannot compensate for omitted/reordered commands, a different
  bank-upload history, timer/echo/envelope phase, or a reconstructed seed.
  Correlated residuals are therefore recorded as unresolved, rather than
  assigned to the emulation-bug bucket by elimination.

The first nonzero native output is pair 164,601, in iteration 199, module
`00/04`, music `$01`. A separate ±160,000-pair search finds its first 2,048-pair
waveform at oracle pair 169,310 relative to source host 81: native is 4,709
pairs (147.15625 nominal ms) earlier. Correlation is 0.99999823, RMS i16 error
2.262 and maximum error 13; the window is not identical. These PCM windows and
search result are retained as `first-audible-{native,oracle}.pcm` and
`first-audible-match.json`. The earlier oracle transient at pair 1,961 must
not be confused with this sustained waveform match.

The full direct policy comparisons retain the changed output and control flow:

| Cartridge versus live policy | Seeded census pair | Uninterrupted pair |
|---|---:|---:|
| Iterations with identical native PCM spans | 173,149 | 165,747 |
| Iterations with different callback counts | 302 | 21,031 |
| Different zero/nonzero poll-observation sequences | 67 | 2,597 |

Observation-sequence differences include changed/missing polls after gameplay
branches, not only different values at the same caller. In particular, the
uninterrupted live run's hold-count differences after quest divergence cannot
be treated as pricing errors for the same work.

The seeded live reconstruction points are 382000, 383636, 464000, 628000,
629845, 736000, 738000, 882000, 884000, 884327, 1066000, 1066963, 1320000
and 1320900. None occurs in the cartridge-answer run or either no-seed run.

Native state at the first sample difference, after iteration 2: SPC absolute
cycle 51,264, local cycle 51,284, PC `$085A`, SP `$CF`, A `$5C`, X `$45`, Y `$0A`,
timer 0 `(cycles=34, divider=0, target=16, counter=0, enabled=true)`, no bank
transfer and all eight native DSP voices inactive. At iteration 200 the clock
is absolute 5,297,280, PC `$0846`, timer 0 `(98,9,16,0,true)`, output port 0
`$01`. Full before/after snapshots, voice states and SPC instructions are
retained at iterations 2, 199 and 200; the binary snapshots include the modern
renderer and driver-clock machine.

The dense-prefix diagnostic used iteration 200 for the retained correlated
state; the regular full-route window schedule first accepts a correlated
residual at iteration 300 (offset −803 pairs locally, playback −4,709 pairs,
correlation 0.99999719, RMS 5.948, maximum sample error 25).

The original frozen host receipts provide output PCM, not matching per-window SPC RAM,
registers, DSP register-write timestamps or internal DSP state. The three
frozen Snes9x savestates do not cover these early mismatch hosts. Consequently
these artifacts cannot establish equal DSP input/state at an aligned residual.
Naming a first SPC/DSP bug from them would exceed the evidence. That was the limit of the initial no-Snes9x-execution phase. The renderless
follow-up below captures both selected windows and establishes unequal input.

## Follow-up: exact elapsed-host accounting (2026-10-25)

The earlier description of “2,474 windows” needs qualification: **2,474 is a
net difference, not 2,474 individually identified skipped physical fields**.
The complete receipt scan finds 1,575,801 accepted NMIs in the route-relative
1,580,998 hosts: 1,473,124 `Open` and 102,677 `LatchHeld`. There is one more
terminal opening than the logical driver's 1,473,123 completed iterations.
398,865 hosts have no acceptance; 393,668 extra acceptances occur in hosts
with multiple acceptances. Thus hosts minus acceptances is 5,197.

Of the zero-acceptance hosts, 393,757 finish a handler accepted in the
preceding host; 5,108 have neither acceptance nor completion and continue a
suspended main call. Group consecutive zero-acceptance hosts around each of
these strict interiors: there are **228 episodes**, containing **5,225 hosts**.
The other **117 episode-edge hosts** also finish the preceding handler. This
keeps actual loader entry/return edges in the span catalogue rather than
mistaking every handler completion for an unrelated normal-frame cut.
The remaining 393,640 zero-acceptance hosts are isolated handler cuts.

| Episode owner / class | Strict interior | Handler-completion edge | All zero-acceptance hosts | Native masked callbacks | Difference |
|---|---:|---:|---:|---:|---:|
| `06/00`, pre-dungeon room/graphics load | 2,163 | 50 | 2,213 | 0 | +2,213 |
| `08/00`, overworld properties / song-bank load | 2,513 | 54 | 2,567 | 2,604 | −37 |
| `11/00`, falling dungeon entrance | 282 | 8 | 290 | 0 | +290 |
| `17/01`, save/quit reset and bank transfer | 69 | 2 | 71 | 72 | −1 |
| `1B/02`, spawn-select room rebuild | 21 | 1 | 22 | 0 | +22 |
| `15/00`, Agahnim mirror-warp setup | 22 | 0 | 22 | 24 | −2 |
| `18/00`, Ganon-emergence setup | 23 | 1 | 24 | 24 | 0 |
| `19/00`, Triforce-room setup | 15 | 1 | 16 | 0 | +16 |
| Subtotal | **5,108** | **117** | **5,225** | **2,724** | **2,501** |
| Isolated handler-cut balance: 393,640 minus 393,668 extra acceptances | | | | | −28 |
| Terminal source opening beyond completed iteration count | | | | | +1 |
| **Exact residual** | | | | | **2,474** |

`source-zero-acceptance-runs.json` lists every complete episode with host
bounds, iteration, owner context and interior count;
`elapsed-affected-iterations.json` additionally retains both the interior and
complete-episode host ranges. `source-boundary-balance.json` lists the signed
isolated-cut/extra-acceptance accounting for every module/submodule, including
intro and polygon contexts; its sum is −28. An equivalent strict-interior
identity is `5,108 + (117 − 28) + 1 − 2,724 = 2,474`. Thus the previously
identified +89 boundary balance includes 117 loader-edge hosts and −28 net
host-cut/extra-acceptance adjustments; it must not hide those loader edges.

Together with the accepted-hold deficit, `5,389 + 2,474 = 7,863` fewer native
callbacks. Equivalently `1,580,998 − (1,473,123 + 102,677) = 5,198` source
hosts beyond one callback per iteration/held NMI; subtract 2,724 native
masked callbacks. The local span quantity `hosts − 1 − lag_nmis` has positive
sum 373,788 and negative sum −368,590, hence net 5,198. Charging every positive
span as a new missing frame would grossly overcount.

The strict elapsed class comprises 4,994 `CallStackContinued`-only hosts, 107
that also complete the common suffix, five that reach sprite preparation,
one with a sprite-main return and suffix, and one with Triforce palette
progress. All are enumerated in `elapsed-no-handler-hosts.jsonl`, grouped
into 228 interior spans in `elapsed-no-handler-spans.json`; the complete
episodes, including the 117 edge hosts, are in `source-zero-acceptance-runs.json`.
`source-no-acceptance-hosts.jsonl` additionally lists **every** zero-acceptance
host, including handler cuts, with source entry module, corresponding native
entry room, iteration and presented blanking. The room field is context from
the native map; it is not a fresh source WRAM read at each masked host. `source-host-summary.json` counts all module contexts;
`elapsed-accounting.jsonl` retains every signed span adjustment. No intro or
boot context contributes a strict elapsed/no-handler host after the host-81
origin. The excluded 81-host reset/upload bootstrap stays separate: all 81 receipts
have empty semantic streams and zero acceptances (`bootstrap-host-proof.json`). Polygon
thread contexts likewise do not create another strict elapsed class: their
handler splits and multiple acceptances are already in the boundary balance.
Do not add another “poly” allowance on top of this census.

All **5,225 complete-episode hosts** have a video SHA-256 identical to their
immediately preceding source host (`episode-presentation-proof.json`), as do
all 5,108 strict interiors (`elapsed-presentation-proof.json`). Thus the
pinned source presents a held surface during every listed interval.
`source-episode-hosts.jsonl` lists the complete episodes host by host.
5,086 strict elapsed hosts carry a fully forced-blank, brightness-zero receipt
with `retain_prior_surface=true`; the 22 `15/00` hosts carry a normal
brightness-15 receipt. Including the 117 edges gives 5,203 fully blank
receipts and the same 22 normal-brightness receipts. These are real elapsed
source presentation/audio
windows even when Zelda never accepts an NMI. The source/hardware cause is
independent clocks: the PPU advances across V225 and ends screen refresh even
when `$4200.7` prevents NMI delivery. Pinned Snes9x `cpuexec.cpp` sets
`SCAN_KEYS_FLAG` at the screen boundary independently of its subsequent
`$4200 & $80` test, and tests pending NMIs before the host-break flag. The
latter ordering explains why an adjacent pair of hosts can have zero/two
acceptances. A ROM load or upload cannot stop the PPU or the SPC clock.

### Specification for the pacing agent (no pacing changes here)

Preserve elapsed physical fields across synchronous main work, including
forced-blank loads and song-bank handshakes while `$4200.7` is clear. Drive
this from the continuous source-owned master clock and retained call phase,
not a table of route iterations, modules, accepted-NMI counts, or the value
2,474. At each crossed presentation boundary, advance audio and publish the
PPU's actual held/blank/current scanout once. An NMI-enabled boundary separately
runs the handler if hardware accepts it; a masked boundary must not run the
handler, sample joypad input, consume RNG, publish new NMI commands, or clear
the software wait latch. Preserve the source's register writes and upload
handshake timestamps within the interval. Resume the suspended CPU work at its
actual clock, carrying fractional phase through adjacent calls and restores.
A later NMI enable during active vblank must retain the real pending-edge
behavior; do not manufacture a full extra field at enable/return.

The 228 complete episodes are an investigation/coverage list, not production rules.
Native already emits 2,724 masked callbacks, including contexts whose net
count exceeds the strict source class; replacing those with an unconditional
additional 5,225 callbacks would double count. Validate continuous field
coverage, ordering, source command times and source handler acceptances
separately. Preserve the chosen 534-pair output budget. Host cuts/terminal
conventions are analysis corrections, not gameplay holds to implement. Apply a
future change only to lagless pacing; default/ROM-timing state and audio
ledgers must remain byte-identical.

## Follow-up: live renderless chip-input investigation (2026-10-25)

These captures use the pinned instrumented Snes9x 1.63 core, serially through
`--run-snes9x-script`, without a GPU, video comparison, or video artifacts. A prefix is replayed
from `oracle_initial.state`, saved outside the cache, and then the short
window is captured with `--audio-trace-dir`. This saves **before-host full
Snes9x states**, raw CPU port accesses, SPC instruction timestamps, every DSP
register store with output-sample/phase coordinates, DSP voice/sample state,
and PCM. All 24 source hosts 385–408 and all 26 hosts 147280–147305 reproduce
the cached PCM byte for byte. The latter ends in `07/00`, room `$043`.
`first-source-cache-check.json` and `worst-source-cache-check.json` retain
those independent checks; the cache was read-only throughout.

Native captures use the same continuous cartridge-answer policy as the
original measurement. Repeated `--audio-state-at ITERATION:DIR` now saves the
returned `AudioEventFrame` (including raw `parity_dsp` stores) as
`CALLBACK-events.json` alongside the existing full before/after audio
snapshots and SPC instructions/voice state. The snapshot includes the retained
SPC RAM/timers, driver and modern DSP renderer; it is not just a WRAM image.
Final capture directories are `task2/first-traced/190..209` and
`task2/route-traced/134690..134710`. Source states are under
`task2/{first,worst}-source/before-HOST.state`. None of these independently
clocked starting states has been shown equivalent.

`ZELDA3_DEBUG_SPC_PORT_TRACE_FRAMES=LO-HI` observes delivered native port
stores at absolute SPC-cycle coordinates, with callback index and delivery
path. It covers nominal scheduled stores, instruction-return delivery, and
retained native CPU-bus accesses. The song-bank transfer owner is separate:
this trace must not be advertised as a complete upload handshake history.
Neither selected window contains a bank transfer. Raw source port timestamps
are `apu_cycle_after` (after synchronization, before the next CPU instruction)
in each host's DSP sample/phase coordinate. Concatenate host sample counts to
compare **relative** times. The instrumented core's instruction absolute-cycle
counter is diagnostic and is not restored by a savestate; do not subtract it
from the native absolute clock. DSP stores use `output_sample*32 + dsp_phase`
in the same per-host coordinate; native stores use
`sample_offset*32 + timer_cycles`. Absolute origins are deliberately independent.

### First audible music: identical command identity, unequal delivery

The unique music command `$2140=$01` anchors native callback **302**
(iteration **193**, before the following iteration's entry-context music byte
changes) to source host **392**. Its following stores are:

| CPU input port / value | Native delivered SPC cycle | Source delta from music store |
|---|---:|---:|
| `$2140=$01` | 5,160,601 | 0 |
| `$2141=$00` | 5,160,601 | 15 |
| `$2142=$00` | 5,160,601 | 21 |
| `$2143=$00` | 5,160,601 | 23 |

Native coalesces these at one instruction return. The first ten store tokens
match after the anchor, but the **11th store already differs**: native clears
port 0 at callback 305, while source host 395 is still omitting that store
and writes port 1. Thus even command order/acknowledgement history is unequal,
not merely the continuous playback origin. See `first-command-alignment.json`.
This observation does not rely on waveform search or the apparent similarity
of silence/clear stores.

The first audible voice-1 key-on (`DSP $4C=$02`) occurs in native iteration
199 at pair **164,584**, phase 4, and source host 398 at absolute source pair
**212,400**, phase 14. The integer-pair displacement is **47,816**; subtracting
the 43,107-pair bootstrap gives the original route-relative 4,709-pair
playback lead. The same event therefore corroborates the earlier waveform
position at native pair 164,601 / source-relative pair 169,310 without blind
waveform search. There is still a **10-SPC-cycle phase difference**.

After aligning that key-on, the following 65 DSP address/value tokens match,
but relative timing first differs at token 7 (`$2D=$00`): native is **32 SPC
cycles earlier**. The matched prefix's relative timing errors range from
−392 to +354 cycles; token 65 then differs in register order (`$6C=$00` in
native versus `$5C=$00` in source). Command-to-key-on time is **106,091**
native cycles versus **111,716** source cycles, a difference of **5,625**
cycles (175.78125 sample periods). These are not identical chip inputs under
an origin shift. The correlation 0.99999823 / RMS 2.262 / max 13 residual
therefore does **not** establish a DSP arithmetic defect. See
`first-keyon-alignment.json` and the raw event/instruction captures.

### Worst correlated residual: different DSP pitch input

At native iteration **134,700**, `07/00`, room `$043`, music `$11`, callback
146,541 delivers all four zero stores at **SPC cycle 2,504,092,629**.
Source host **147,293** delivers the same four values at per-host coordinates
54, 64, 73, 73: relative offsets **0, 10, 19, 19**, versus all zero in native.
The 26-host source window has no nonzero CPU port command; repeated clears
cannot identify an independent event. The new CPU-command matcher correctly
returns **unresolved** here (`worst-command-alignment.json`).

A narrower DSP-note check pairs `$4C=$40` in that native iteration (sample
offset 8, phase 29) with the source host's offset 505, phase 8. Sixteen
preceding DSP address/value stores agree in reverse order, but the **very
next DSP store differs**: voice 2 pitch-low `$22` is **`$DA`** in native after
1,337 SPC cycles, versus **`$E9`** in source after 1,324 cycles (source host
147,294). This is both a **15-unit register-value difference** and a
**13-cycle relative timing difference**, directly upstream of synthesis.
`worst-keyon-alignment.json` retains the exact store coordinates. This local
note pairing is not a uniquely resolved CPU-command identity: repeated
key-ons can be different notes, and the matcher leaves that identity
unresolved. A DSP renderer receiving these different pitch bytes is not an
equal-input emulator test;
the RMS 3,477.21 residual cannot be assigned uniquely to BRR, interpolation,
envelope, echo, mixing or another arithmetic component.

Both selected comparisons therefore fall in **Task 2(b): unequal
pacing/transport/driver history**. This rules out using these residuals as
proof of an equal-input SPC/DSP defect; it does not prove every SPC/DSP
operation correct. No emulation arithmetic fix, injected source state, or
pacing change was made. An actual component regression requires a separately
controlled equal-state/equal-timed-input replay. The available live captures
now make that boundary explicit rather than treating correlation as proof.

### Correspondence method and fixed-budget effect

`lagless_audio_compare.py commands NATIVE_LOG SOURCE_TRACE_DIR OUTPUT` now
anchors a unique nonzero CPU port event first, then compares subsequent raw
store order and relative SPC-cycle timing. It rejects all-zero/repeated-only
identities. In the **two investigated probes**, command-event identity resolves
**1/2 (50%)**; equal-timed-input probes resolve **0/2**. The first audible
position is additionally corroborated by its voice key-on. The worst window
has no fresh unique CPU command, and its nearby note has different pitch
input. These two-probe percentages are not full-route coverage statistics.

No full-route CPU-command timestamp trace exists in these captures, so the
new method cannot truthfully upgrade the other waveform windows. Full-route
resolved coverage remains **3,857/25,384 = 15.1946%**, and the previously
measured sample-identical-after-alignment count remains **22 usable windows**
(27 including ambiguous best shifts). No new full-route aligned-audio census
or increase in those counts is claimed: Task 2(a)'s emulation-fix condition was
not met. The trace-enabled native prefix is byte-identical to the original
cartridge PCM for **413,843,456 bytes**, including both investigated windows;
the separate early traced prefix also matches byte for byte. See
`route-prefix-pcm-prefix-proof.json` and `first-traced-pcm-prefix-proof.json`.

The independent 534-pair design choice is unchanged. Source average is
**533.122002684 pairs/host** on the covered route, so 534 is **0.164689754%**
larger. Its exact duration contribution remains **+1,388,112 pairs
(+43.3785 nominal seconds)**, separately from **−4,198,842 pairs** due to
callback-count differences. There is also a transport consequence to inspect:
`host_port_target_cycles` computes nominal bus targets from physical NTSC
fields, while output advances by 534 pairs per callback. A target that has
fallen behind the output/execution cursor is delivered at the current
instruction boundary; several ports can consequently coalesce, exactly as
observed above. Do not “repair” the deliberate output budget to make an
emulation comparison pass. A transport/pacing fix must explicitly preserve
which clock owns each command and how its phase maps into output time.

Reproduction uses the core and ROM under the original checkout's
`external/snes9x-libretro/local/` and `zelda3.sfc`, read-only. The short source
captures start from the saved `task2/source-385.state` and
`task2/source-147280.state`, with `--input-frame-offset 385` / `147280`,
24 / 26 frames and `--audio-trace-dir`; their preceding serial script runs
start from the cache's initial state and apply its exact `input.txt`.

## Follow-up verification

Default and reduced parity binary builds are warning-free. With
`ZELDA3_ASSETS=/Users/missingno/Documents/zelda3-rs/zelda3_assets.dat`, the final
Zelda default suite reports **2,509 passed, one known spotlight caller-return
failure, three ignored**; reduced reports **2,044 passed, zero failed, two
ignored**. Both SNES library suites report **504 passed, zero failed, seven
ignored**. The eight Python comparison/command-identity tests pass. The
existing default library-test compilation warnings remain; no new library
warning or ordinary binary-build warning is claimed fixed.

The final instrumented default binary matches every record and EOF of the
independent clean-main 200k v2 ledger (696.840 seconds). Main advanced during
the follow-up, so a second clean `git archive` build of main
`36610d133b56bd26e778657f6c6c8292a98f58b0` was made in
`/tmp/zelda3-audio-main-36610`, using the same ROM/assets. That binary also
matches every record and EOF of the same ledger (504.765 seconds). This
checks the final observer against both the retained main baseline and the
newer main revision without importing the pacing agent's changes here.
See `task2/default-200k-final.log`, `task2/current-main-200k.log` and
`task2/current-main-check.json`.

The final full cartridge-answer logical census **passes** at all
**1,473,123 iterations**: 737 WRAM-clean segments, no panics, no unexplained
state/RNG/command mismatch, and zero non-initial reseeds. Its 14 classified
lag-state differences remain separately reported. Held callbacks **97,288**
and masked callbacks **2,724** are unchanged, hence total callbacks remain
**1,573,135**. See `task2/census.log` and `task2/census-gate.log`:
`census gate: PASS (unexplained=0, complete=True)`. This is a state/RNG/command
census with continuous native audio execution, not a new full-route
aligned-waveform measurement.

`task2/followup-verification.json` binds the final report and observation
sources, separate default/reduced/main binaries, complete test/build/ledger/
census logs, both source-window traces and boundary states, native snapshots,
command/register alignments, and exhaustive elapsed-host lists. The original
`artifact-manifest.json` remains the identity of the initial capture phase.

No GPU runs/tests, video flag, cached A/V run, oracle-cache writes, formatting,
commits or pushes were performed. Snes9x core runs were serial and headless;
their output artifacts contain audio and machine state. Pacing and DSP/SPC
arithmetic are unchanged. Scratch analysis drivers and the main source
archives stay in `/tmp`.

## Initial regression verification

Both final parity binary builds completed without warnings. Default Zelda
library tests: 2,509 passed, 1 known spotlight caller-return failure, 3 ignored.
Reduced Zelda library tests: 2,044 passed, 0 failed, 2 ignored. Each SNES suite:
504 passed, 0 failed, 7 ignored. Default library compilation emitted the existing
24 warnings; the ordinary binary builds emitted none. The reusable Python
comparison tests cover signed offsets, sample residuals, silence, repeated-wave
ambiguity, cache write protection and missing holds. Default and ROM-timing
engine/library sources and instruction prices are unchanged.

The independent clean-main 200k default state ledger completed in 765.843 s;
the capture binary matched every record and EOF in 734.464 s (exit 0).
The final binary, including the optional census shortcut, separately matched
every record and EOF in 370.595 s (exit 0; `ledger-compare-final.log`).
This includes v2 state, display, SRAM, PCM, input and context, using the same
pinned source inputs. See `clean-main-ledger.log`, `ledger-compare.log`,
`clean-main-default-200k.jsonl.zst` and its metadata. Binary artifacts are
separate from the build output paths, so subsequent feature builds cannot
replace the binaries used for these runs.

In that initial phase, no GPU runs/tests, video flag, cached A/V run, Snes9x execution, oracle-cache
writes, formatting, commits or pushes were performed. Scratch drivers and the
clean-main source archive stayed in `/tmp`; repository additions are the
reusable comparison tool, its tests, this report and replay observation flags.

## Initial evidence identities and remaining limits

`artifact-manifest.json` binds the capture binary, final replay source,
analysis script, supplied iteration ledger, all four PCM/map/CSV/log sets,
summary JSON, extracted oracle PCM/index and the pinned source hashes.
`verification.json` binds the separate final default/reduced binaries, both
warning-free final build logs, library outcomes, six Python tests, the final
200k comparison, the full cartridge repeat and the plot hash. All four
post-processing comparisons and both direct policy comparisons exited zero;
each audio summary requires exactly 1,473,123 contiguous iterations.

The primary measured objects are native DSP-grid PCM and logical-driver
histories. Nominal-millisecond conversions are not a measurement of a user's
audio device, its resampler or wall-clock scheduling. The source-host mapping
is not a sample-accurate NMI cut; the alignment search is finite and sampled;
most windows have no reliable match. The remaining unfinished analysis is a
validated catalogue of actual sound-event onset identities and isolation of
any SPC/DSP arithmetic/emulation defect under identical input and state.
No emulation fix or gameplay/pacing change is claimed.

## Lagless port transport experiment on main 8e15ac9d (2026-10-25)

The before binary was built from a clean `codex/spc-port-transport` checkout
at `8e15ac9df65d2e25d6870d5a7bca3c99492e6297`, before any edits.
New evidence is under `target/spc-port-proof/`; the older measurements above
remain evidence for their original binary, not this experiment's baseline.
In particular, current-main iteration 134,700 begins at native pair
78,260,904 / callback 146,556, versus 78,252,894 / callback 146,541 in the
older capture. Both map to source host 147,293. No pacing source was edited.

### Clock ownership and candidate mapping

`HostPortTransport::frame_writes` reconstructs the nominal NMI's conditional
music/ambient branches and unconditional effect stores. These stores do not
carry a retained native CPU-bus timestamp. Their existing offsets come from
`host_port_master_clock_offsets`, in CPU master clocks relative to nominal
NMI entry. `host_port_target_cycles` previously assigned their *epoch* from
physical NTSC fields, even though lagless output advances 534 pairs per
callback. That mismatch eventually puts the entire batch behind execution.
The instruction-step/return scheduler then coalesces overdue events.

Other owners are separate. Native `cpu_port_at` / `cpu_main_port_at` use
`ApuHostPortTiming` and its retained master-clock conversion/credit. Main
semantic writes without a raster timestamp retain their end-window boundary.
`SongBankHostTransfer` owns the upload command, ready/ack polls, headers,
bytes and clears, with its own master-clock cursor; upload-return NMIs retain
their existing timestamped owner. The candidate leaves all these paths alone,
and uses the old complete path for any callback with an active bank transfer.

For a nominal lagless NMI batch, let `M` be the existing nominal NMI entry,
`d[p]` its CPU-path offset, `C(m)=floor(m*15664/328125)`, and
`E=max(output_window_start, executed_SPC_cursor)`. The candidate maps:

`T[p] = E + C(M + d[p]) - C(M)`.

This is one translation of the batch, not per-port deadline clamping or
stretching CPU instruction durations. It preserves each CPU-derived cycle
difference and the existing fractional conversion phase. The callback epoch
follows the deliberate 534-pair clock, so physical-field drift cannot put the
batch in the past. Events enter the APU's existing cycle-level input queue
before any new SPC execution, where they latch before a same-cycle bus read.
They are not rounded again to an opcode/pseudo-op return. Queued events survive
output-window boundaries and the 32-bit local-cycle rebase. No new serialized
state, output sample budget, SPC/DSP arithmetic, instruction price, gameplay
hold rule, source value injection or route-specific rule is introduced. The runtime
entry point is selected only by `lag_free`; ordinary `advance` is unchanged.

The CPU master gaps and Snes9x's synchronized delivery gaps are distinct.
Fresh renderless trace-core runs reproduce all 24 first-window and 26
worst-window PCM buffers from the previous independently cache-verified
captures. The core hash is
`6f112667f7b590c3f789630171ce9af79b1793245ad2e14080522146ffd6d498`.
Source music stores have relative CPU master times 0/324/408/470, but
synchronized SPC times 0/15/21/23. Worst-window clears have CPU master times
0/178/262/324 and synchronized SPC times 0/10/19/19. Synchronization may finish
an SPC pseudo-op and carry positive credit; its receipt times therefore need
not equal a direct rational conversion of CPU bus gaps. The candidate fixes
the nominal transport; it does not claim equal initial state or identical
Snes9x coroutine synchronization.

### Fresh chip-input probes

| Probe | Before native relative SPC cycles | Candidate | Source synchronized cycles |
| --- | --- | --- | --- |
| First music, callback 302 / source 392 | 0/0/0/0 | 0/15/19/22 | 0/15/21/23 |
| Iteration 134,700, callback 146,556 / source 147,293 | 0/0/0/0 | 0/9/13/16 | 0/10/19/19 |

Music port0 moves from cycle 5,160,601 to 5,160,619. Both before and after
match the same first ten CPU store tokens and diverge at the 11th: native
clears port0 at callback 305 while source writes port1 at host 395. The first
relative timing disagreement moves from token1 (-15 cycles) to token2
(-2 cycles). Thus independent delivery improves without repairing the older
acknowledgement/history difference.

First voice-1 key-on remains in iteration 199, at native absolute cycle
5,266,692 before and 5,266,688 after. Both match 65 subsequent DSP address/value
stores; the first relative timing error remains token7, -32 cycles, and
register order diverges at token65. Timing-error ranges are -392..354 before
and -384..362 after. This is not an equal-input arithmetic comparison.

The worst probe remains command-unresolved in both captures: all CPU stores
are repeated zeros. In the fresh current-main baseline and candidate, the
source's nearby `$4C=$40` does not occur inside native callback 146,556, so
the older note pairing cannot be carried forward to this binary. All 102
native DSP stores in this callback are identical before/after, including
relative timing. No repaired pitch input or newly resolved note is claimed.
`source-bus-spacing.json`, `chip-probes.json`, the command alignment JSON,
raw source traces and native event/snapshot files retain these boundaries.

### Full-route cartridge-answer audio comparison

Both fresh captures contain all 1,473,123 contiguous iterations and zero
actual reseeds, so their audio execution is continuous despite allowing
`--seed-every 2000`. The source stream and comparison method above are
unchanged. The current-main before column replaces neither the older binary's
results nor their provenance; it is a new baseline.

| Measure | Clean main before | Candidate |
| --- | ---: | ---: |
| Sample-exact iterations | 657 | 658 |
| Nonzero sample-exact iterations | 606 | 607 |
| Resolved waveform windows / attempts | 3,826 / 25,384 | 3,805 / 25,384 |
| Sample-identical after alignment, usable windows | 18 | 21 |
| Identical at a best shift, including ambiguous windows | 32 | 29 |
| Native callbacks | 1,573,272 | 1,573,266 |
| Held callbacks | 97,421 | 97,421 |
| Masked callbacks | 2,728 | 2,722 |
| Native stereo pairs | 840,127,248 | 840,124,044 |
| Source stereo pairs | 842,864,820 | 842,864,820 |
| Final native-minus-source pairs | -2,737,572 | -2,740,776 |

The six-callback reduction comes from different SPC execution entering the
unchanged bank handshake, not a new hold/field rule. The first callback-count
difference is one *additional* masked callback at iteration 306,911;
later transfers yield the net reduction. Every callback still emits 534 pairs.
The net 3,204-pair shortening is 100.125 nominal milliseconds and is not a
source-pacing repair. Equal-length exact iterations increase by one and usable
identical windows by three, while resolved coverage decreases by 21 windows.
These mixed finite-search results do not establish equal chip input, corrected
DSP arithmetic, or full-route audio equivalence. Summary and per-window files
are `before-rom-summary.json[.windows.jsonl]` and
`after-rom-summary.json[.windows.jsonl]`.

### Census, gameplay and default-mode regression

Both cartridge and `--live-apu-reads` gates pass, before and after, over all
1,473,123 iterations: `PASS (unexplained=0, complete=True)`. Cartridge has
737 WRAM-clean segments and live has 742, with no panics, ordinary RNG/APU-read
mismatches, or unmatched ordinary APU commands. Cartridge engine/source command
totals remain 67,990 / 67,290; live remains 67,939 / 67,287.

| Live-SPC census measure | Before | Candidate |
| --- | ---: | ---: |
| Held callbacks | 97,314 | 97,314 |
| Missing holds | 5,956 | 5,956 |
| Excess holds | 593 | 593 |
| Zero-source-lag calls with a native hold (FP) | 133 | 133 |
| Masked callbacks | 2,701 | 2,693 |
| Actual reseeds | 14 | 14 |
| Classified song-poll WRAM differences | 640 | 640 |

The 14 live reseeds occur at identical iteration positions. Both modes retain
14 classified lag-state differences; live also retains 14,741 classified
polygon-state differences and seven classified song-poll command differences.
These remain distinct from zero *unexplained* mismatches. The live census is
seeded and is not an uninterrupted same-quest or same-audio proof. The fresh
before live run with predictor output and the separate run without it have
identical 2,462 census records (`before-live-observer-check.json`). No census
classification or matching rule was changed to obtain these results.

The default/ROM-timing nominal entry point remains ordinary `advance`; lagless dispatch
comes after the retained real-CPU audio-boundary path. The capture binary and
final rebuilt default binary both match every record and EOF of the independent
clean-main 200k v2 ledger. That baseline's metadata records the requested
`8e15ac9d` head, `git_dirty=false`, and the actual clean-main binary hash.
`after-default-ledger.log` and `final-default-ledger.log` retain the two checks.
The final rebuilt binary additionally reproduces the capture's first 210
lagless iterations byte for byte in PCM and mapping (`final-prefix-check.json`).
Only a scheduler comment/diagnostic source position and test bodies changed
after full capture; no runtime function body changed. The measured full-route
binary and the final build have separate hashes in the manifest, and no new
full-route execution of the latter is claimed.

The complete live predictor comparison finds 12 iterations with different held
counts and 16 with different masked counts. Module/submodule contexts agree on
every row. The changed SPC/upload phase also changes some downstream raster,
HDMA and cost receipts; identical aggregate holds must not be advertised as
identical per-call timing. `live-row-comparison.json` retains changed-column
counts and first examples. No instruction prices or pacing implementation were
edited, and neither the requested census gates nor aggregate excess/FP/missing
metrics worsened. The candidate is retained on that bounded evidence.

### Tests, artifacts and reproduction

Both final ordinary parity builds are warning-free. Default Zelda library:
2,515 passed, only the known
`spotlight_caller_return::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`
failure, three ignored. Reduced: 2,050 passed, zero failed, two ignored.
Both SNES suites: 504 passed, zero failed, seven ignored. Final focused
transport/upload suites pass all 35 tests in each feature mode. The default
library-test compilation retains the 24 pre-existing warnings; no new warning
or ordinary binary-build warning is hidden by that distinction.

The four new tests check CPU-derived spacing/fractional phase and monotone
adjacent callbacks; before/at/after ordering against actual SPC input reads;
independent latching, pending events across output boundaries and a local-cycle
rebase; and byte-identical bank-owner state through ready/ack, payload and
terminal clear phases under an identical controlled receiver. That receiver
is a unit fixture, not a replacement cartridge workload or parity proof.
The complete Zelda suites ran with four test threads from the final test
executables. The SNES results and final focused suites have separate logs.

`verification-summary.json` collects exact counts and gates;
`artifact-manifest.json` binds the clean-main, capture and rebuilt binaries,
the candidate patch, measurement PCM/maps, source and native probes, source
inputs, build/test/ledger/gate logs and this report. The capture default binary
is `after-zelda3`; the final rebuild is `final-default-zelda3`. Source and test
bodies are retained in `candidate.patch`. Scratch orchestration stayed in
`/tmp`. The superseded serial library attempts are not acceptance evidence;
the sandbox rejected attempts to stop those processes, and their logs are
separate from the completed four-thread suites.

Reproduction uses the cache/ledger paths defined above. Run the before and
candidate binaries with `--logical-replay "$L" "$C/initial.srm"
--seed-every 2000 --report 0 --audio-pcm PCM --audio-map MAP`; use separate
outputs and `--live-apu-reads --pacing-predictor CSV` for live gameplay metrics.
Compare each cartridge capture with
`scripts/lagless_audio_compare.py compare ORACLE PCM MAP SUMMARY
--expect-iterations 1473123`, and gate each census with
`scripts/logical_census_summary.py LOG --gate --expect-iterations 1473123`.
Probe port traces use the existing `ZELDA3_DEBUG_SPC_PORT_TRACE_FRAMES` gate,
with repeated `--audio-state-at` captures as above. Fresh source probes use
`--run-snes9x-script`, the pinned trace core, saved before-host source states,
`--input-frame-offset`, and `--audio-trace-dir`, serially and renderless.

No GPU runs, video flag, cached A/V runs, cargo GPU tests, oracle-cache writes,
formatting, commits or pushes were performed. The 534-pair budget is unchanged.
This removes nominal lagless port coalescing while preserving CPU-derived
spacing; it does not repair the first command-history mismatch or establish
source-equivalent chip input/audio.

## Lagless sound proof 1

Measurement at `3bd00a96`, branch `codex/sound1`, using `full-v2` and the
read-only `ed0121e1...` cache named above. Runtime, prices, pacing, classifiers
and tables are unchanged. All new evidence is under `target/sound1/`.
No GPU, Snes9x process, live-APU census, continuous live replay, full library
suite, formatting, commit or push is part of this batch.

### The two native sample grids

The older fixed-chip-budget description above does **not** describe this
base. `audio.rs::zelda_render_audio` now advances
`spc_driver_clock.rs::LaglessSampleClock` by 533 or 534 native samples, then
`resample_lagless_callback` interpolates that output into the caller's buffer.
The logical harness requests **534 returned pairs per callback**. Its PCM and
mapping therefore describe resampled playback output, not raw chip samples.
The current clock's cumulative generated-pair count after `C` callbacks is

`F(C) = floor((C * 1364 * 262 - floor(C / 2) * 4) * 15664 / (328125 * 32))`.

This distinction is necessary for both duration and waveform claims. A
post-resampler mismatch cannot isolate an SPC or S-DSP defect. Nor does
equality of port values in a short window establish equal chip state: RAM,
timers, DSP phase, voice/envelope state and echo history must also agree.

### Original authority and first-music chip boundary

The fresh cache scan verifies **all 1,581,079 PCM hashes**, **842,907,927
stereo pairs**, and both manifest-bound receipt/hash-ledger SHA-256 values.
The route-relative origin remains host 81: **842,864,820 pairs** in
**1,580,998 hosts**, with **43,107 bootstrap pairs** excluded explicitly.
See `source-proof.json`.

The retained `lagless-audio-notes3/original-full` DSP capture is supplementary
source evidence, not a newly executed oracle. Its **1,581,079 audio hash rows**
were independently rechecked against this cache. The fresh native first
voice-1 key-on is callback **308**, logical iteration **199**, SPC clock
**5,266,688**; the source key-on is host **398**, clock **6,796,814**. Native
DSP event coordinates are corrected from the observer's 534-pair callback
base to `F(callback)` before comparing clocks.

After aligning those key-ons, **65 ordered DSP address/value writes match**.
The first relative timing difference is write **7**, `$2D=$00`, **32 SPC
cycles early** natively. Write **65** differs in identity/order: native
`$6C=$00`, source `$5C=$00`. Current first-music port deliveries have relative
SPC clocks **0/15/19/22**; the earlier source capture documented above has
**0/15/21/23**. The source delivery clocks are prior capture evidence, not a
fresh source-instruction trace. `chip-prefix.json` retains the exact boundary.
This is an **unequal-input finding**, not an equal-input emulation failure.

### Full-route length accounting

The fresh ROM-answer run completes **1,473,123 iterations**, with **zero
actual reseeds**, **100,279 held callbacks**, **5,138 masked callbacks**, and
**1,578,540 total callbacks**. The ordinary census gate passes with zero
unexplained differences; that gate's adjacent-iteration matching and
classifications are not strict command-identity proof.

| Sample domain | Native pairs | Native minus route-relative source |
| --- | ---: | ---: |
| Returned, resampled harness PCM | 842,940,360 | **+75,540** |
| Generated chip samples, from the verified fractional clock | 841,554,405 | **−1,310,415** |

Thus the old **−2,740,776-pair** result is not the result at this base.
The chip-count deficit is **40.95046875 seconds at 32 kHz**. The returned
stream is longer by 75,540 pairs because it expands every callback to 534
pairs; its length is not the elapsed chip clock. The returned-minus-generated
conversion contributes exactly **1,385,955 pairs**.

Every returned pair is accounted for below. Each callback-count term uses
the harness's actual 534 returned pairs. Elapsed episode labels are mapped
source module/submodule contexts, not newly instruction-traced producer PCs.

| Exact term / owner context | Source episode hosts | Native masked callbacks | Returned pair contribution |
| --- | ---: | ---: | ---: |
| Missing accepted holds: 102,677 − 100,279 = 2,398 | — | — | −1,280,532 |
| Excess accepted holds | — | — | 0 |
| `06/00`, pre-dungeon load/upload | 2,213 | 2,196 | −9,078 |
| `08/00`, overworld load/upload | 2,567 | 2,498 | −36,846 |
| `11/00`, falling dungeon entrance | 290 | 290 | 0 |
| `15/00`, Agahnim mirror-warp setup | 22 | 24 | +1,068 |
| `17/01`, save/quit reset and bank transfer | 71 | 68 | −1,602 |
| `18/00`, Ganon-emergence setup | 24 | 24 | 0 |
| `19/00`, Triforce-room setup | 16 | 15 | −534 |
| `1B/02`, spawn-select rebuild | 22 | 23 | +534 |
| Isolated handler cuts versus multiple acceptances: 393,668 − 393,640 | — | — | +14,952 |
| One terminal source opening beyond completed iterations | — | — | −534 |
| Returned 534-pair grid versus actual source host PCM lengths | — | — | +1,388,112 |
| **Total native minus source** | **5,225** | **5,138** | **+75,540** |
| **Unaccounted residual** | | | **0** |

There are **228 complete elapsed episodes**, with **5,108 strict interiors**
and **117 handler-completion edges**. The exact remaining elapsed-host balance
is `5,225 − 5,138 − 28 + 1 = 60`, not an unnamed transport allowance.
Together with 2,398 missing holds, there are **2,458 fewer native callbacks**.
Boot and polygon/thread scenes are not extra allowances: boot is excluded
explicitly, and the route's timing-defined contexts are already included in
the hold and handler-boundary counts. Adding a second poly correction would
double count them. `duration.json` retains every context and all 228 episodes;
`source-boundaries.jsonl.zst` retains every source host's acceptance/completion
counts.

On the chip grid, `F(1,580,998) = 842,864,819`, exactly **one pair below** the
source route's stored PCM. Therefore the independent chip-count identity is
`−1,310,414` from the callback-count difference, plus **−1** from the source
origin/fractional phase, yielding **−1,310,415**, with zero residual. Including
the source's excluded bootstrap instead gives **+32,433 returned pairs** or
**−1,353,522 generated pairs**. These are origin choices, not additional
runtime losses.

### Full-route command identity, order and iteration

The delivery-observed ROM run also completes all 1,473,123 iterations. Removing
only `spc_port_write` lines leaves census logs **byte-identical** to the PCM
capture. It observes **5,826,128 actual native port stores**: 5,825,444 through
`lagless-cycle`, 381 through `instruction-return`, and 303 through `scheduled`.
These are stronger observations than queue snapshots, but the existing switch
does **not** trace the complete bank-transfer handshake/payload path.

The comparison retains zero clears, normalizes nonzero transitions per port,
and uses the existing bounded monotone matcher unchanged. Source upload
payloads are retained separately, using explicit IPL headers rather than a
write-count cutoff. Its **67,643 source transitions** and **68,154 delivered
native transitions** give:

| Observation class | Count | First occurrence |
| --- | ---: | --- |
| Matched content/order under the bounded alignment | 67,510 | source/native iteration 0 |
| Source-only normalized transition | **133** | iteration 33,678, port 3 `$0C` |
| Native-only delivered normalized transition | **644** | iteration 3,333, port 3 `$0C` |
| Same-iteration equal multisets with different order | **39 iterations** | iteration 12,404: source `(1,$05),(2,$19)`; native reversed |
| Matched transition outside raw iteration deltas `{−1,0}` | **73** | source 34,318 / native 34,319, ambient `$05` |

**The strict zero-difference target is not established.** These classes are
different views of the observations, not disjoint audible-error counts.
Repeated identical commands do not have unique event IDs. Mixed insertion,
deletion, reordering and iteration shifts can produce the same alignment;
the 39 order-only iterations do not prove that all other reordering is absent.

All unmatched value classes and their first occurrences are retained below.
Port/value includes the pan bits; no low-six-bit equivalence is substituted.
“Owner” names the observed production boundary to investigate, not a proven
individual source producer or an assertion that an audible sound was lost.

| Port / value | Source only | Native only | First source / native iteration | Owner boundary |
| --- | ---: | ---: | --- | --- |
| 0 / `$1B` | 4 | 0 | 167310 / — | Music request / acknowledgement / rearm |
| 1 / `$05` | 109 | 3 | 48556 / 370546 | Ambient request / acknowledgement / upload return |
| 2 / `$0E` | 1 | 1 | 625275 / 625274 | Timed SFX1 request and NMI clear |
| 2 / `$1E` | 2 | 0 | 878130 / — | Timed SFX1 request and NMI clear |
| 2 / `$2A` | 1 | 1 | 625286 / 625285 | Timed SFX1 request and NMI clear |
| 3 / `$02` | 4 | 0 | 624607 / — | Timed SFX2 request and NMI clear |
| 3 / `$0C` | 3 | 634 | 33678 / 3333 | VWF glyph / retained marker / NMI clear |
| 3 / `$10` | 1 | 0 | 1443256 / — | Timed SFX2 request and NMI clear |
| 3 / `$1E` | 1 | 0 | 961739 / — | Timed SFX2 request and NMI clear |
| 3 / `$24` | 2 | 2 | 434088 / 434087 | Timed SFX2 request and NMI clear |
| 3 / `$42` | 2 | 0 | 624667 / — | Timed SFX2 request and NMI clear |
| 3 / `$59` | 1 | 1 | 1257884 / 1257884 | Timed SFX2 request and NMI clear |
| 3 / `$66` | 1 | 1 | 1257903 / 1257886 | Timed SFX2 request and NMI clear |
| 3 / `$A6` | 1 | 1 | 1257910 / 1257893 | Timed SFX2 request and NMI clear |

All source `$FF` upload-request edges find matches in the delivered inventory;
there is no unmatched `$FF` row here. This does not establish equality of the
unobserved native upload payload/handshake. Queue-publication observation alone
reported **271 source-only / 644 native-only** transitions and 67,372 matches:
the delivery observer resolves **138** additional source transitions. The
publication-only result must not be used as a missing-hardware-command count.

Raw iteration deltas below are **native minus source**. The common −1 is the
documented semantic-main/closing-NMI versus source opening-NMI convention,
not 63,982 independently proven scheduling bugs. A global +1 relabeling is
also insufficient: held NMIs and within-iteration publication need their own
owners. Exact source/native first examples remain in `delivered-identity.json`.

| Delta | Matches | First source / native iteration |
| ---: | ---: | --- |
| −4 | 25 | 56238 / 56234 |
| −1 | 63,982 | 194 / 193 |
| 0 | 3,455 | 0 / 0 |
| +1 | 3 | 34318 / 34319 |
| +2 | 5 | 625234 / 625236 |
| +3 | 1 | 354594 / 354597 |
| +4 | 2 | 354595 / 354599 |
| +5 | 9 | 354596 / 354601 |
| +6 | 3 | 354602 / 354608 |
| +7 | 1 | 354600 / 354607 |
| +9 | 8 | 354606 / 354615 |
| +10 | 12 | 354609 / 354619 |
| +13 | 1 | 625221 / 625234 |
| +16 | 2 | 1257886 / 1257902 |
| +18 | 1 | 625322 / 625340 |

Without any iteration relabeling, 117,522 event-bearing iterations have
different content/multiplicity, with 62,778 source-only and 63,289 native-only
same-index tokens; 4,456 event-bearing iterations have identical ordered
lists. These intentionally strict raw-index totals retain the convention
difference rather than hiding it. They are not counts of missing/extra sounds.
`delivered-commands.jsonl.gz` retains every matched and unmatched event,
including native delivery SPC clock and source timing bounds.

Those source timing fields are the existing analyzer's **opening-host
estimates**, not independently verified bus timestamps. In particular the
first-music ledger anchor is host **391**, whereas the retained source bus
capture places its store in host **392**. A handler can cross the host cut.
Consequently even an initial NMI group's point estimate must not be treated
as a source delivery-clock proof. No matching/classification rule was changed
to conceal this limitation.

### Command-anchored waveform windows

The complete delivered-command alignment supplies **67,510 window attempts**,
representing **64,441 distinct native/source window pairs**; commands in the
same callback can share a window. Each native window contains **2,048 returned
pairs**, beginning **4,096 returned pairs after the delivery callback's start**.
The source search is centered the same number of pairs after its opening-host
anchor, with a **±3,200-pair** integer-shift search. This is a 64 ms window
after a nominal 128 ms delay on a 32 kHz grid, not an audible-span detector;
it can miss an already-ended short SFX. Source delivery clocks remain absent.

The unchanged correlation helper requires score ≥0.95, an interior winner,
and no competing/plateau ambiguity. Equality is a separate byte comparison.
Both complete PCM streams are consumed and hashed, without materializing an
uncompressed full stream on disk. Native PCM SHA-256 is
`a77981ad8401070ba555ee37a8fe3a6e6c6e5d8a1130c0608b4d7d5a5c96ece1`;
source concatenated PCM SHA-256 is
`ecca4f7d1d1fd533f9697a4ad9f8510516a3c95774b48946d42fa1b16680533c`.

| Window outcome | Count |
| --- | ---: |
| Usable correspondence | **3,261 (4.83%)** |
| Weak correlation | 63,958 |
| Ambiguous or search-boundary match | 48 |
| Flat native window, alignment unresolved | 243 |
| **Byte-identical among usable windows** | **0 / 3,261** |

Flat windows are not counted as proven equal sounds. The finite search and
low resolved coverage prevent extrapolating the usable fraction to all
sound events. Among usable windows, RMS i16 errors have minimum **0.1105**,
5th percentile **58.37**, median **332.56**, 95th percentile **1,218.51**,
and maximum **2,278.27**. No usable window is sample-exact. This compares
resampled native output with source chip PCM, so it is not a raw-DSP fidelity
score.

| Source upload cohort | Attempts | Usable | Identical usable |
| --- | ---: | ---: | ---: |
| Boot, before first recorded transfer | 462 | 106 | 0 |
| Dungeon payload `81b19e3a…` | 46,741 | 1,503 | 0 |
| Overworld payload `4aca3a1b…` | 19,657 | 1,633 | 0 |
| Ending payload `4b08fad4…` | 650 | 19 | 0 |

Cohorts use the most recent **source transfer start**; they include transfer
windows and do not certify native residency or bank-byte identity. By command
kind, usable/attempted counts are music requests **286/526**, controls
**98/801**, ambient **300/2,442**, SFX1 **1,483/38,041**, and SFX2
**1,094/25,700**. These label the anchor command, not the voice responsible
for every sample in the mixed output.

The five largest usable residuals below have distinct window coordinates.
They are not a ranking of audible defects, and no equal-input SPC instruction
or DSP defect has been isolated at them.

| Source / native iteration | Source module, room, music | RMS i16 error | Maximum sample error | Correlation |
| --- | --- | ---: | ---: | ---: |
| 1412271 / 1412270 | `07/00`, `$06C`, `$16` | 2,278.27 | 10,079 | 0.956251 |
| 1449272 / 1449271 | `07/00`, `$000`, `$1F` | 2,093.97 | 12,720 | 0.962287 |
| 138661 / 138660 | `07/00`, `$033`, `$15` | 2,071.73 | 8,317 | 0.960762 |
| 402252 / 402251 | `09/00`, `$10F`, `$09` | 2,054.59 | 8,637 | 0.950721 |
| 414064 / 414063 | `07/00`, `$038`, `$16` | 2,010.07 | 8,357 | 0.951792 |

The first music request's window has correlation **0.967087**, RMS **214.66**,
and maximum error **524**, but is **ambiguous** because a competing peak is
within 0.001. Its independent first-key-on DSP diagnosis above is stronger
evidence than selecting that waveform peak.

The demonstrated differences to investigate are the command histories,
delivery phase, 2,458 omitted callbacks, and the output resampling grid.
Their individual shares of waveform error are **not identifiable from these
observations**; inherited timer/voice/echo state is not controlled. Remaining
residuals are explicitly unisolated, not assigned to emulation by elimination.
`wave-summary.json` and `wave-windows.jsonl.gz` retain the complete distributions,
all attempts, exact checks and worst windows.

### Proof boundary and concrete next batches

**Equal-input chip equivalence and full-span per-sound equality are not
established by this batch.** The first-music window already has unequal DSP
input. The other windows do not have paired, equal SPC/DSP starting states
and complete source port-delivery clocks. Three source bank-transport payload
identities cover 228 uploads: dungeon **113**, overworld **114**, and ending
**1**, plus the initial boot segment. Their presence in a waveform cohort is
not proof that native uploaded the same bytes at the same chip clocks.

The fraction of sound events whose **entire audible span** is identical is
therefore **unmeasured**, not zero. A command is not a labelled voice lifetime:
music and SFX overlap, a channel can be stolen or retriggered, and a release
can leave an echo tail. The cache's mixed PCM cannot uniquely recover those
owners. A correlated window, or even an identical short window, cannot fill
that missing denominator. No SPC/S-DSP arithmetic defect is inferred from an
unaligned or differently driven window.

The concrete follow-on work, in dependency order, is:

1. **Close the command observation boundary.** Extend the existing diagnostic
   observation of `spc_driver_clock.rs::SongBankHostTransfer` to retain bank
   identity, every transfer/clear store and its delivery clock. Retain the
   actual opening/held/closing-NMI owner with each native request, rather than
   globally shifting every event by one iteration. Audit
   `zelda_rtl/rtl_native_pacing.rs::{native_time_apu_request,
   native_publish_apu_requests,native_checkpoint_sound_queue}` and
   `nmi.rs::interrupt_nmi_audio_parts` at the reported first discrepancies.
   Expected result: distinguish a genuinely lost/reordered request from an
   upload-path omission or an iteration-coordinate mismatch, then repair the
   responsible publication phase once. Do not change the census exclusions
   to make the count zero.
2. **Resolve dialogue and acknowledgement ownership at delivery.** The click
   producer is `messaging.rs::begin_vwf_glyph`; the retained publication owners
   are `audio.rs::ModernAudioCommandQueue::{push,pop}` and
   `spc_driver_clock.rs::HostPortTransport::{frame_writes,nmi_latch_writes}`.
   Follow each reported `$0C` edge through the producer, zero clear, queued
   marker and actual SPC store before deciding whether it is an extra audible
   click. Other SFX values go through the same timed request/NMI boundary;
   the trace does not identify every individual sprite/ancilla producer.
   Expected result: remove only demonstrated duplicate/lost/reordered
   deliveries, preserving legitimate clear-and-repeat requests and pan bits.
3. **Restore elapsed fields from source-owned work.** Continue the existing
   missing-cost work through
   `rtl_native_pacing.rs::{native_cost_components,native_finish_main_cost,
   run_native_held_frame}`. Audit masked work through
   `run_native_song_upload_frame`, `native_mask_song_upload_nmi`, the
   `cycle_models/song_bank.rs` model and `SongBankHostTransfer`, using the
   episode table above. Restoring all 2,398 missing holds would add exactly
   **1,280,532 returned pairs**; closing the remaining net 60-field gap adds
   **32,040 returned pairs**. On the fractional chip clock, restoring all
   2,458 callbacks adds **1,310,414 generated pairs**, leaving only the
   one-pair origin/phase difference. These are aggregate count consequences,
   not permission to add a route/module-specific hold table.
4. **Build equal-state, equal-input chip fixtures before an emulation fix.**
   Observe native samples immediately before
   `audio.rs::resample_lagless_callback`; retain SPC RAM, registers, timers,
   pending deliveries, DSP phase/voices and echo state. Pair them with
   independently cache-verified source instruction/state windows for first
   music, the worst retained residuals, and each bank. Compare the first
   differing instruction, DSP write and raw sample in that order.
   `SpcDriverClock::advance_lagless`, its port scheduler, and
   `modern_audio::ModernAudioEngine` are the corresponding owners. The current
   source artifacts do not supply those equal-state fixtures. Fix delivery
   phase if inputs differ; name an emulation repair only after equal state
   and input produce a first divergent operation.
5. **Then establish sound lifetimes.** Track command acceptance to voice
   ownership, KON/KOFF, envelope release, steals and echo tails. Compare those
   raw rendered spans under the controlled chip fixtures; compare playback
   output only after applying the same resampling grid and history to both
   sides. This is what would make the requested per-sound fraction meaningful.

While lag holds remain absent, **whole-stream equality on the original chip
clock is unreachable**: the measured streams have different lengths and
inter-command spacing. A translation of each window cannot repair that
spacing or the resulting overlap of music and effects. Logical command
identity and conditional chip-emulation equality remain useful independent
targets, but neither establishes the original continuous soundtrack.

### Verification and retained evidence

Default and `--no-default-features` parity builds are warning-free. The
focused two-million-callback clock, resampling-history and audio-snapshot
tests pass; their library-test compilation retains the existing 24 warnings.
The existing timing/comparison Python suites pass **16 tests**, and six
scratch measurement tests cover overlapping windows, truncation, ordering,
complete-stream hashing and the accounting identities. No full library suite
was run.

The 420-iteration diagnostic-on/off comparison is byte-identical for **PCM,
predictor CSV and census logs** (`observer-transparency.json`). The separate
port observer also reproduces that prefix PCM. Source and binary identities
are in `provenance.json`, the corrected ledger hash in `ledger-binding.json`,
and the retained source-DSP manifest checks in `retained-source-binding.json`.
The default capture binary is retained compressed as `zelda3.zst`.

`run-command.json` and `ports-command.json` retain the exact ROM-only
invocations. `pcm.zst`, `map.zst`, `predictor.zst`, `timing.jsonl.zst`, and the
compressed delivery log retain the observations; derived reports retain
their conventions separately. Scratch analysis stays in `/tmp/sound1-*.py`;
no one-off script was added to the repository. An earlier capture stopped at
the internal 2.85 GB guard near iteration 1.46 million. Its hashes and stop
reason are in `interrupted.json`; its incomplete capture files were removed,
and it is not counted as a completed run. The replacement starts from the
beginning and completes with exit zero and all compressor exits zero.

`verification.json` binds the retained artifacts, scratch analysis and report.
Final evidence occupies **2.954 GB**, below the 3 GB cap. `git diff --check`
passes; the only tracked change is this report. This is a verified measurement
stopping point, not completed strict sound-equivalence proof.
