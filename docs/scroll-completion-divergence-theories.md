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
