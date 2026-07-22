# Scroll-completion single-frame divergence — theory log

**Symptom:** continuous route (routes/clean) diverges on exactly ONE frame per
message line-scroll: 1391, 1491, 1543, 1661, 1761, 1861, 2083, 2180, 2283,
2401, 2447, 2535, 2667 (13 total). Committed baseline `0ede92c9` = 17 total
ranges (these 13 + the 2826..2837 fast-forward residual + 2875).

**Precise observation (frame 1391, first scroll):**
- Video: 1389,1390 both top=151; 1391 rust top=151 / oracle top=170; 1392 both
  top≈169. The text makes a DISCRETE ~19px jump (a line scrolls off). The
  oracle jumps at 1391; rust jumps at 1392 — one frame late.
- rust scroll passes (0x1cdf) match the oracle frame-for-frame (2,2,1 per
  group; verified). So the SCROLL TIMING matches; only the DISPLAY of the
  completion is off by one frame.
- Pixel-level: oracle's 0x7c00 at 1391 (Y) = the final buffer (Z) shifted back
  one row (Y[i] = Z[i-1]).
- Text-CHR (0x7c00) DMA uploads happen only at scroll-GROUP boundaries (frames
  1386, 1389, 1392, 1393 — PC 0x008d0d, 2016 bytes), NOT every frame. rust
  uploads 0x7c00 every frame.
- No writes to BG3 v-scroll WRAM shadow (0x00ea) during the scroll (checked
  trace). BUT the direct PPU register $2110 (BG3VOFS) is NOT covered by the
  WRAM watch — UNVERIFIED whether the ROM writes it directly per frame.

## Theories TRIED and RULED OUT (do not repeat)

1. **Present live on the group-completion (final lag) frame** instead of the
   frozen pre-scroll buffer. `dialogue_scroll_ran_this_frame = !is_final_lag`.
   RESULT: WORSE — 17 -> 43 ranges. Each scroll gained extra diverging frames
   (e.g. 1384, 1387, 1390..1391). Regressed the group-START frames of every
   scroll, not just fixed the completion. Reverted. (Tested twice — not a
   flake; full-route confirmed.)

2. **Greedy-fit budget for the fast-forward render** (yield BEFORE a glyph that
   overflows). Unrelated to scroll but logged: made the 2826 residual WORSE
   (rp_mismatch 7 -> 10-13). Reverted. (This is the OTHER residual.)

3. **Frozen from WRAM buffer 0x10000 vs frozen from VRAM 0x7c00** — both tried
   earlier in the session; the current committed model freezes 0x10000 (the
   ROM's 7F:0000 DMA source), verified by per-upload checksums. Works for
   12/13 frames per scroll.

## Theories NOT yet tried

- **A. Direct PPU BG3VOFS ($2110) per-frame write.** The ROM may do the smooth
  sub-jump via the v-scroll REGISTER (not the 0xea shadow), uploading the CHR
  only at group boundaries. Rust bakes the whole scroll into the CHR. NEXT:
  read the oracle's actual BG3 v-scroll register (not WRAM shadow) per frame —
  needs a PPU-register probe (debug_ppu_value or extend DisplayPpuProbe). If it
  changes per frame, reimplement the scroll to use BG3 v-scroll for sub-tile
  motion + CHR upload only at group (8px) boundaries.
- **B. Harness memory-read-vs-display frame offset.** Repeatedly hit: the
  instrumented core's zelda3_vram_trace_frame and the harness frame index and
  oracle_vram(N) read point may be offset by one relative to the DISPLAYED
  video. Resolve by anchoring on a frame where the display provably changes and
  correlating oracle_vram, the video, and the DMA trace. If oracle_vram(N) =
  display(N+1), several contradictions above dissolve.
- **C. rust's every-frame 0x7c00 re-upload is the bug.** Make rust upload
  0x7c00 only at group boundaries (skip nmi_upload_bg3_text during scroll lag
  frames), so rust's DISPLAYED CHR is piecewise like the oracle. The frozen
  presentation approximates this but from the WRONG generation on the
  completion frame.

## RESEARCH FINDINGS (walkingeyerobot/alttp-disassembly vwf.asm)

- **VWF_Scroll ($74FE2) shifts the tile buffer in RAM ($7F0000)** via
  `.moveTileUpOnePixel` (LDA $0002,X : STA $0000,X ...). **It writes NO BG3
  scroll register.** => Theory A (v-scroll register) RULED OUT. Same mechanism
  as rust (buffer shift).
- On line completion (moved 0x10 px): `LDA $1CD9 : ADD #1 : STA $1CD9` (advance
  read_pos past [Scroll]); `LDA #$0050 : STA $1CDD` (reset VWF render X to
  0x50); `STA $0722`/`$0720` row state; `STA $1CE6`. Returns RTS.
- **VWF_Scroll does NOT request the upload ($14).** The text CHR upload is done
  by the per-frame message NMI. Text_MessageHandler runs once/frame, renders,
  then `LDA #$02 : STA $17 : STA $0710` (pending_nmi_subroutine + core_update_
  disable). The .epsilon budget check is `LDA $1CDD : CMP #$0063 : BCC .alpha`
  (render while VWF X < 0x63=99) and the char decode is
  `LDA $7F1200,X : AND #$7F : SUB #$66` (>=0x66 => command). So the fast-forward
  budget relates to $1CDD reaching 0x63 (99) — NOT my empirical 0x20; revisit
  the 2826 residual with $1CDD semantics.

## Theory D (MOST LIKELY, from research) — one-frame vblank display lag
The ROM uploads the scrolled CHR at frame N's vblank; it is DISPLAYED on frame
N+1's active scan (standard SNES vblank lag). rust bakes the scroll into the
CHR and its compose displays the buffer WITHOUT that one-frame lag, so rust's
displayed scroll runs ONE FRAME AHEAD of the oracle. The committed frozen
model over-corrects (holds the call-frame buffer for the whole group);
"present live on completion" under-corrects (showed the too-current buffer).
FIX TO TRY: present rust's 0x7c00 text region from EXACTLY ONE FRAME AGO during
the scroll (a 1-deep delay line of ppu.vram[0x7c00..0x7ff0]), replacing the
frozen call-frame capture. This models the vblank display lag uniformly.
Regression guard: full 0-3300 must not exceed the 17-range baseline.

## More theories TRIED and RULED OUT (this session, post-research)

4. **NO_FROZEN (present the natural snapshot 0x7c00 during scroll).** RESULT:
   MUCH worse — 1386..1391 ALL diverge (610-719 px each). Confirms the frozen
   is essential: rust's natural pipeline scrolls the CHR smoothly every frame
   while the oracle's displayed CHR is static between group uploads.

5. **One-frame-delayed frozen WINDOW** (stale_scanout(N)=ran(N-1)). RESULT:
   1386..1390 clean, 1391 WORSE (632 -> 983). Delaying the window keeps the
   frozen over 1391 with mismatched content. Wrong direction.

## KEY REMAINING BLOCKER — frame-numbering / shift-count ambiguity
Every reconstruction of "oracle displayed shift-count per frame" contradicts
another, because THREE frame indices are in play and their offsets are
unresolved: (a) harness retro_run/video frame N, (b) instrumented-core
zelda3_vram_trace_frame, (c) the SNES vblank-vs-active display phase. The
pixel fact is solid: oracle DISPLAY(1391) = Z shifted back one row (Y[i]=Z[i-1]),
where Z = rust's fully-scrolled buffer. rust's frozen shows the group-START
(pre-scroll) buffer at 1391; rust's LIVE buffer at 1391 = Z (too far). NO rust
state equals Y at any frame under the current scroll shift-then-increment.

6. **Delayed-LIVE buffer** (present WRAM 0x10000 from one frame ago during
   scroll). RESULT: worse — 1386,1387,1389,1390 diverge (610-904). Ruled out.
   => Theory D FULLY RULED OUT (window-delay, no-frozen, and live-delay all
   worse than the committed frozen). None of rust's buffer generations
   (frozen call-frame, live current, live one-back) equals the oracle's Y at
   the completion frame — so Y is NOT a simple generation/timing shift of
   rust's buffer. The scroll BUFFER SEQUENCE itself likely differs at the
   completion (shift-then-increment order, or the completing pass 0x10 timing).

NEXT (needs a NEW measurement, not more of the same): capture, in ONE run, the
DISPLAYED 0x7c00 (compose snapshot) AND the exact shift count for BOTH rust and
oracle keyed to the SAME frame index, e.g. by having the trace core stamp the
video-frame index it is producing. Then the shift-count-per-displayed-frame is
unambiguous and the correct presentation (which rust buffer generation to show)
falls out. Candidate fix once resolved: present rust's LIVE 0x7c00 from exactly
one frame ago during scroll (a 1-deep delay line) — models the vblank lag on
the live buffer rather than freezing the call-frame buffer. (Delayed FROZEN
window failed; delayed LIVE buffer is UNTESTED and is the top candidate.)

## Research needed (ALTTP decomp / online)
- The exact US ALTTP routine for message VWF line scroll and its per-frame
  VRAM upload + BG3 scroll behavior. Routine names to find: the message
  scroll ("RenderText_Draw_Scroll"-equivalent), Hud/BG3 v-scroll handling
  during messages, and whether it uses $2110 BG3VOFS.

7. **Refresh frozen to current post-pass buffer on the FINAL lag frame.**
   RESULT: worse — 1387, 1390 break (1084, 983), 1391 unchanged (632). Like all
   frozen-timing changes, it CASCADES to adjacent frames because the frozen is
   a single shared state across groups. => Any per-frame frozen tweak regresses
   neighbors. The frozen model gets 12/13 frames right by the call-frame capture
   coincidentally matching the oracle's group-boundary uploads; the completion
   frame needs a state outside that model, and adjusting it breaks the balance.

## CONCLUSION (this session)
7 theories tested, ALL regress the committed baseline (0ede92c9, 17 ranges).
The single-frame-per-scroll residual is NOT fixable by incremental frozen-timing
tweaks — it needs either (a) resolving the 3-way frame-numbering ambiguity via a
new single-run measurement that stamps the produced video-frame index into the
instrumented trace, or (b) replacing the shared-frozen presentation with a
per-group independent model that cannot cascade. Committed baseline kept.

## SOLVED — commit 1145cb59
The winning approach (theory 8): a DEDICATED completion override, separate from
the shared frozen state, presented with a ONE-FRAME STAGE DELAY. On the
group-completion (final lag) frame, capture the current scrolled WRAM VWF
buffer (7F:0000) into `dialogue_scroll_completion_pending`; stage it one frame
(`_staged` -> `_text`) so it displays on N+1, matching Snes9x's vblank scan-out.
Keeping it separate from the frozen avoided the cascades that killed theories
1,5,6,7. The one-frame delay was the key: the earlier "present live/completion
on frame N" attempts landed one frame early (rust top jumped at video-N while
the oracle jumped at N+1). Result: all 13 per-scroll divergences gone; route
17 -> 4 ranges. Verified frame-by-frame by text top-edge.

## SESSION 2026-07-21 — DEFINITIVE REFRAME: full-route cascade root is intro-telepathy animation-phase drift, NOT a graphics bug

**Method:** added per-frame both-sides WRAM dumps via `ZELDA3_DEBUG_WRAM_FRAMES`
(harness writes `rust_wram_frame_N.bin` + `oracle_wram_frame_N.bin`). Diffed
meaningful bytes (excluding 0x00-0x1f zero-page + 0x100-0x1ff stack) across the
route.

**Findings (committed budget=0x20 baseline, Rust vs Snes9x oracle):**
- The continuous route's video ranges: `2826..2837, 2875, 14661, 14663..37842`
  (basically the entire tail from 14661 diverges).
- **14661 is NOT a graphics-load / CHR decompression bug.** Screenshot compare:
  same room 0x0104 (Link's house), the only visible diff is the BED (top-left) —
  oracle shows the detailed/animated bed tile, rust a plain one. VRAM first
  mismatch = word 0x3B00 = the DUNGEON ANIMATED-TILE region (nmi.rs:269). Trace
  at 14661: `link_tile src=B2C0 cd=7` (rust) vs `B280 cd=4` (oracle) — Link OBJ
  tile-upload ~3 frames out of phase; `0xc8`(select r16)=0x00 vs 0x0d;
  `animated_link_tile_dma_source`(0xae0)=0xa0/0xc0 vs 0x80. All ANIMATION-PHASE,
  not content.
- **WRAM is ~1680 meaningful bytes divergent from ~frame 1350 onward and NEVER
  re-converges** (1350→14661 all ~1680). Video matches 2876..14660 because the
  divergent bytes live in non-rendered regions (page 0x17000 gfx-decompress
  scratch 848, 0x1D000 mapbak 240, 0xE000 294, 0x1000 283). At 14661 one of
  them (the dungeon animated-tile phase) finally reaches the screen.
- **The persistent seeds:** `0x0710` (core_update_disable) = **0x02 oracle /
  0x00 rust** for frames 1350..2700; `0x00c8` (5-way alias: menu-anim-timer /
  intro-sword-ypos / R16 / ...) = **nonzero & counting (0x0a→0x0d) oracle /
  0x00 rust** from 1350. i.e. during the INTRO TELEPATHY the oracle holds
  core-update and ticks a message/animation timer that rust does not, so rust's
  frame/animation counters drift a few steps ahead. That phase offset is
  invisible until the bed animates at 14661.
- At frame 500 `0xc8` and `0x710` still MATCH (0x00/0x00, 0x80/0x80); by 1000
  they diverge (c8 1e/cb, 710 80/06). Root window = ~500..1350 (first intro
  telepathy line render). meaningful diff: 500→887, 1000→2892, 1350→1680.

**2826 fast-forward budget — RULED OUT approach (this session):**
- ROM `VWF_RenderRecursive` (max speed) renders one glyph then `LDA $1CDD :
  CMP #$0013/#$003B/#$0063 : BEQ yield`. I mapped $1CDD → rust's
  `vwf_glyph_advance_prefix_sum(glyph_cursor)` (pixel prefix-sum) and yielded on
  `== 0x13|0x3b|0x63`. **REGRESSED** rust from 2 chars ahead (read_pos 0x0d vs
  0x0b) to 14 ahead (0x19 vs 0x0b). Reason: **$1CDD is NOT the pixel prefix-sum**
  — it is a "line position in the tilemap buffer" counter (per RAM.log +
  disassembly), init 0, stays 0 during INITIAL FILL (frames 2825-2827:
  oracle 0x1cdd=0 while read_pos climbs 5→18), only becomes 0x28 once SCROLLING
  starts (2828). rust's per-line line_x is 0-based per line and OVERSHOOTS the
  boundaries (hit 124, 158). So the 19/59/99 check is scroll-phase only and
  does not model initial fill. Reverted to budget=0x20.
- Oracle initial-fill read_pos advance (chars/frame): +6,+7,+7,+5,+8,+7 (~40px/
  frame ≈ 0x28, not the 0x20 budget). rust (budget 0x20) is 2 chars ahead at
  2826 — but this is a MINOR contributor; the 1680-byte divergence predates it.

**CONCLUSION / NEXT:** the true root is byte-exact frame-stepping of the intro
telepathy (core-update-hold + the 0xc8/animation timer), frames ~500..2800, NOT
the 2826 budget and NOT any graphics load. Fixing 2826 alone cannot converge the
tail. Need: make rust hold core-update and tick the message/animation timers
frame-identically to the oracle through the first telepathy so 0xc8/0x710 and
the animated-tile phase match. Tool: `ZELDA3_DEBUG_WRAM_FRAMES=<csv>` +
`--session-dir` dumps both sides' WRAM for byte diffing at any frame.

## SESSION 2026-07-21 (cont.) — PRECISE ROOT: BG_TILE_ANIMATION_COUNTDOWN over-decrements during intro telepathy (0x710 hold not replicated)

Pinned the 14661 visible bed divergence to ONE driver byte:
- `0xc00d` **BG_TILE_ANIMATION_COUNTDOWN** = rust 0x03 / oracle 0x05 (off by 2).
  It gates the dungeon BG animated-tile phase (VRAM 0x3b00 = the bed tile). The
  frame counter `0x1a` MATCHES (0x31 both) — so this is NOT frame-counter drift;
  it is a SEPARATE countdown decremented a different number of times.
- Decrement site: `nmi_prepare_sprites` (misc.rs:1443) `if
  self.decrement_bg_tile_animation_countdown() == 0 { reset to 9 (or 0x17) }`
  — also advances `animated_tile_data_source_address` (0xa680+off) on reset, and
  the sibling `link_dma_countdown` (0xc013, off by 3 at 14661) + `0xae0`
  link_tile_src. All the 14661 visible diffs are these animation phases.

**Mechanism (definitive, via trace core PC attribution):** during the intro
telepathy (module 0x0E, frames ~850..1174) the ROM writes 0xc8 from PC $00E781/
$00E79E and sets `$0710` (NMI_DISABLE_CORE_UPDATES) = 0x02 each message frame
(disasm: Text_MessageHandler ends `LDA #$02 : STA $17 : STA $0710`). With
`$0710`!=0 the NMI SKIPS the core-update section — including the BG-tile /
Link-DMA animation-countdown advance in nmi_prepare_sprites. **Rust's `$0710`
stays 0x00 through the telepathy** (dump: fr1200 rust 710=0 / ora 710=2), so rust
runs nmi_prepare_sprites and decrements the countdowns on frames the oracle
holds → rust's BG_TILE_ANIMATION_COUNTDOWN ends 2 lower, and the animated-tile
phase (bed) is 2 steps off, surfacing at 14661 and cascading the tail.

Trace-core recipe (PC-attributed WRAM writes to 0x000-0x3FF + 0x800-0x810):
```
sha=f8c5bb640d1fad3d74fa0e4439a1c44ff2fce29adb65fc944542c80a226ea655  # trace dylib
ZELDA3_SNES9X_VRAM_TRACE=/tmp/wt.txt target/parity/zelda3 --compare-snes9x-oracle \
  external/snes9x-libretro/local/snes9x_libretro_trace.dylib saves/zelda3.sfc <N> \
  --expected-core-sha256 $sha --expected-rom-sha256 <romsha> \
  --input-script routes/clean/comparisons/continuous/continuous-input.txt \
  --load-sram routes/clean/comparisons/continuous/initial.srm \
  --compare-from-frame 0 --ignore-audio --session-dir <sd> --scan-all
# then: grep " wram 00c8 " /tmp/wt.txt  → "<frame> wram <addr> <val> <PC>"
```

**FIX (next session, not yet applied):** replicate the ROM's `$0710`=2
core-update hold during the intro-telepathy message-render frames so rust SKIPS
the nmi_prepare_sprites animation-countdown decrement on exactly the frames the
oracle holds. rust's hold currently gates the MAIN core update (`skip_run` /
`dialogue_fast_forward_hold_active` in Module0E_Interface) but NOT the NMI
animation-countdown advance (it never sets 0x710). Care: the intro currently
renders mostly-correct video; validate 0-3300 range count does not regress and
that BG_TILE_ANIMATION_COUNTDOWN + link_dma_countdown match the oracle at 14661
(both-sides `ZELDA3_DEBUG_WRAM_FRAMES=14661`).

## SESSION 2026-07-21 (cont. 2) — fix landed (partial) + remaining root pinned to intro poly-step phase

**Applied fix (zelda_rtl.rs:9607):** gate `nmi_prepare_sprites()` on `!hold_core`
(same flag as the frame counter). ROM-faithful (matches the $0710 NMI
core-update skip). game_state tests: 300 passed. Effect: BG_TILE_ANIMATION_
COUNTDOWN (0xc00d) telepathy drift off-by-2 → off-by-1; 0xae0 link_tile_src now
matches. **Video full-route ranges UNCHANGED** (still the whole tail) because a
SEPARATE off-by-1 remains.

**Remaining root — intro poly-step hold phase is 1 frame off (PRE-EXISTING, not
caused by the fix):** per-frame both-sides 0xc00d dump (`ZELDA3_DEBUG_WRAM_
FRAMES=795..842`) during module 0 / submodule 5 (the intro, `1a` frame counter
MATCHES, `$0710`=0x80 both): BOTH sides hold the 0xc00d decrement every 3rd
frame, but the phase is offset by exactly 1 — oracle holds at 796,799,802,805…
while rust holds at 797,800,803,806…. So rust's dungeon BG animated-tile phase
is permanently 1 step off → the 14661 bed and the entire tail diverge. This is
the intro poly-step-hold cadence (`rom_intro_poly_thread_is_active` = mod0
sub 3/4/5/7/9/11; fields `snes9x_hold_intro_step_this_frame`,
`snes9x_intro_step_hold_alternate`, `poly_job_hold_frames` in zelda_rtl.rs) —
the known-hard poly-intro sub-frame timing (see memory
poly-intro-bsnes-residual). nmi_prepare_sprites (0xc00d decrement) is skipped on
the frames the intro poly thread holds; rust's hold phase is 1 frame behind the
oracle's.

**NEXT:** correct the intro poly-step hold phase so rust holds 0xc00d on the
same frames as the oracle (align `snes9x_hold_intro_step_*` / `poly_job_hold_
frames` cadence to the oracle at module 0 sub 5). Verify with
`ZELDA3_DEBUG_WRAM_FRAMES=795..850` that rust's 0xc00d == oracle's every frame,
then the 14661 bed + tail should converge. Definitive per-frame both-sides
0xc00d dump is the measurement.

## SESSION 2026-07-21 (cont. 3) — ROOT TRACED TO A SINGLE BOOT FRAME (~97)

Traced the 0xc00d phase offset to its origin by per-frame both-sides dumps
walking backward: sub5(752+) carries it, sub4(700-751) carries it (both
decrement every frame, rust already +1), sub3(300) already +1, sub1(108-135)
CONSTANT rust=0xfffe / oracle=0xffff — i.e. a SINGLE spurious extra decrement
before frame 108, not a cadence. Pinned it:

- Boot 0xc00d trajectory (harness frames): power-on garbage (rust 0x0000→…,
  oracle 0x5555 pattern) → rust tileset-init resets 0xc00d to 0xffff at harness
  ~95, oracle at harness ~96 (rust's init is ~1 frame early) → at **harness 97
  rust decrements 0xc00d to 0xfffe while the oracle holds at 0xffff** → both
  then hold (constant) until the first real room reset. That one extra
  core-update permanently offsets the animation phase; the first room
  `reset_bg_tile_animation_countdown(9)` inherits the 1-frame-early phase, so
  0xc00d stays 1 off for the whole run and surfaces at 14661.
- Confirmed via `ZELDA3_DEBUG_BOOTGATE` (temporary, removed): at the boot frames
  all early-return gates are false (`pending_rom=false, dlg_init=0,
  hold_core=false`), so rust reaches nmi_prepare_sprites whenever
  `zelda_run_game_loop` is called (run_what & RUN_MAIN). The extra decrement is
  a `run_what`/thread-cadence difference at the boot transition — rust schedules
  one RUN_MAIN frame where the oracle runs RUN_POLY (holds). Frame-numbering:
  bootgate `frame_ctr_dbg` is offset ~1 from the harness WRAM-dump index.

**This is the known-hard poly-intro sub-frame thread-scheduling timing**
([[poly-intro-bsnes-residual-is-irq-thread-timing]]): the main/poly thread
split at boot is driven by cycle-level IRQ timing (crystal_rotation_counter
accumulates VIRQ; `advance_crystal_rotation_counter`), and rust's boot cadence
runs one extra RUN_MAIN frame vs the oracle. A safe fix needs the boot thread
cadence to match the oracle cycle-for-cycle at frames ~80-100; blind tweaks risk
the currently-correct intro video. NOT attempted (no-guessing).

**Status:** landed the correct nmi_prepare_sprites `!hold_core` gate (removes the
telepathy over-decrement, off-2→off-1). The remaining off-1 is this boot-frame
root. Full-route video ranges unchanged until the boot cadence is matched.

## SESSION 2026-07-21 (cont. 4) — DISASSEMBLY-CONFIRMED: unfixable without cycle-accurate boot NMI cadence

Pinned the single extra decrement to harness ~97 and checked the exact mechanism:
- Frame counter 0x1a MATCHES both sides and increments on the same frames
  (every other host frame during boot). $0710=0x80 BOTH sides throughout
  (so NOT the gate). At harness 96 both 0xc00d=0xffff; at harness 97 BOTH
  increment 0x1a (a main-update frame on both), yet rust decrements 0xc00d
  (0xffff→0xfffe) while the oracle holds. The oracle's `0x12` (NMI-done flag)
  ALTERNATES 0/1 during boot then settles; rust's is constant 0x01.
- WW on 0xc00d: every write is the DEC (display.rs:5819); no reset write — the
  native field wraps through 0xffff. rust decrements once per game-frame.
- **Disassembly (Bank00.asm `Main_PrepSpritesForNmi`):** `LDA $7EC00D : DEC A :
  STA $7EC00D : BNE .ignoreTileAnimation` — the decrement is **UNCONDITIONAL**
  (no gate on module / $0710 / graphics-loaded). So there is NO gate to add.
  The divergence is purely that the ROM does NOT RUN `Main_PrepSpritesForNmi`
  on the harness-97 frame (partial NMI / poly-thread interleave) while rust does
  (full NMI). rust's scheduler has `nmi_thread=false` during boot sub1 and runs
  RUN_MAIN every frame; the oracle runs a partial NMI on that frame.

**Conclusion:** the root is the boot NMI cadence — whether each vblank runs the
full core-update (`Main_PrepSpritesForNmi`) or a partial NMI. On real hardware
this is decided by whether the main thread finished its frame before vblank
(cycle-level CPU-workload timing) / the poly-thread interleave. Rust's
frame-granular model runs the full core-update one extra time at ~frame 97. A
correct fix needs cycle-accurate boot NMI-thread cadence
([[poly-intro-bsnes-residual-is-irq-thread-timing]]) — NOT a gate, NOT a
targeted tweak. Confirmed via the ROM disassembly (no-guessing). The landed
`!hold_core` gate (telepathy over-decrement) stays; it is correct but the
remaining off-by-1 is this boot-cadence root, which blocks the 14661→37842 tail.
