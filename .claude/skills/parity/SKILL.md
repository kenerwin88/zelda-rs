---
name: parity
description: Drive the next Snes9x route-parity divergence to a clean, committed root-cause fix — or a committed-nothing precise diagnosis. Use when working on route parity, divergence hunting, or the parity front.
---

# Parity front: routes → 100% Snes9x parity, clean code only

We are driving zelda3-rs toward 100% parity with the pinned Snes9x 1.63 oracle,
verified through the authoritative human route in `routes/full_run` via
`scripts/snes9x_route_recorder.py compare-route --project routes/full_run`
(the `--project` value is the directory path, not the bare name) and the
`scripts/full_parity.py --with-snes9x` cold-boot gate.
The pre-commit parity gate is also self-ratcheting (`scripts/precommit_snes9x_parity_gate.py`) and keeps a persistent checkpoint in
`.git/precommit-snes9x-parity-state.json`; each clean run should move that
checkpoint forward unless you explicitly reset it.

Your job this session: pick up the current parity front, find the next
divergence, root-cause it, and land a **proper fix** — or, if the root cause
resists, leave behind a precise diagnosis and a clean tree instead of a
half-fix.

## Workflow

1. Run the route comparison to find the earliest current divergence
   (`compare-route --project routes/full_run`; add `--no-build` if the release
   binary is already current). It writes a full `replay.sh` into
   `routes/full_run/comparisons/continuous/` — copy it and shrink the frame count
   for a fast repro of an early divergence (note the input script it references
   is named `continuous-input.txt`).
   GPU comparisons run strictly serially — never run cargo GPU tests alongside
   a compare; the binary's flock enforces this.
   For this session, always run the pre-commit gate first and confirm the next
   ratchet target (`pre-commit: Snes9x parity gate target=...`) so you start
   investigation from the current front, not frame 0. If this run improves the
   frontier, immediately rerun pre-commit again with no extra args before
   touching any fix artifacts. Even when an explicit target is set for a focused
   repro, any pass that increases `last_checked_frame` must still be treated as a
   ratchet milestone and committed immediately.
   Do not run any C-oracle scripts or C-checkpoint bisectors in this lane.
2. Use checkpoint resume (`--save-state-at` / `--load-state`) for any probe
   beyond a few thousand frames — never re-replay from frame 0 repeatedly.
3. Root-cause with the standard toolchain in order:
   `ZELDA3_ASSERT_NATIVE_COHERENT` first for "inputs match but the action
   diverges", then `find_dual_ownership.py`, `whoowns.py <addr>`,
   `ZELDA3_WW_ADDR` write-watchpoints, `ZELDA3_WW_BACKTRACE`.
4. Classify the bug into one of the four known classes (stale bulk-projection
   clobber, oversized table, bounded-read-vs-raw-RAM, missing write/branch)
   and apply the matching recipe from CLAUDE.md. If it fits none of them, say
   so explicitly and describe the new class before fixing.

## Hard rules for what lands in the tree

- **Experiments are disposable; only finished fixes are committed.** Temporary
  `eprintln!`s, env-gated probes, hack branches, and throwaway scripts are fine
  while investigating — but before committing, every one of them is removed
  unless it's a genuinely reusable diagnostic (env-gated, off by default,
  documented in CLAUDE.md style).
- No `// TODO: figure out why`, no commented-out alternatives, no magic-number
  band-aids that make the diff pass without an explained mechanism. A fix must
  come with a one-paragraph root-cause explanation naming the address, the
  owner struct, and the bug class.
- Semantic writes touch only the bytes they own — never bulk-project a range
  shared with another system. Make one struct the sole owner and redirect all
  readers/setters; don't just delete the duplicate field.
- Every candidate fix is confirmed with **one from-scratch (non-checkpoint)
  run** before committing (the checkpoint restore artifact at byte 0x654 has
  produced false confidence before), plus
  `cargo test --profile parity -p zelda3 game_state` and a clean
  `find_dual_ownership.py`. For refactors, byte-compare pre/post
  `ZELDA3_REPLAY_WRAM_DUMP` dumps.
- One root cause per commit, with the frame number and address in the commit
  message (matching the existing `f<frame>` convention).
- Never `git checkout <file>` — the user works this repo concurrently;
  surgically revert only your own edits, and use `--no-verify` only for docs.

## Improve the process as you go

Fixing the divergence is only half the job — each session should also make the
NEXT divergence cheaper to find and fix. While working, actively watch for:

- **Friction you hit twice.** If you repeated a manual grep chain, hand-decoded
  bytes, or re-derived something a script could print, that's a tool gap. Small
  reusable diagnostics (env-gated, off by default) and script improvements are
  first-class deliverables — the WW backtrace, coherence checker, and
  `whoowns.py` all started this way.
- **A bug that fits no known class, or a recipe that needed adapting.** Update
  the "Common bug classes" section in CLAUDE.md (or this skill) with the new
  class or refined recipe, so classification stays a lookup instead of a
  rediscovery.
- **Blind spots in the static finders.** If `find_dual_ownership.py` or the
  coherence checker MISSED the bug you just fixed, ask why — extending the
  finder to catch that shape is often worth more than the fix itself.
- **Slow probe loops.** If an investigation step took minutes when it should
   take seconds (missing checkpoint, re-replaying from 0, serial steps that
   could be one instrumented run), fix the loop before grinding through it.

- The parity lane checks only **rust-vs-snes9x libretro** behavior. Remove or ignore
  any non-snes9x oracle/checker references while working in this lane.

## Ratchet discipline

After any passing parity run that moves the known-good frontier, do not stop
there: rerun the self-ratcheting pre-commit gate immediately once more so the
front advances again. If you skip this second run, the next commit/checkpoint can
revisit the same divergence window.

- Every frontier bump is a commit boundary candidate: once a second run confirms
  the same-or-higher `last_checked_frame`, commit that milestone immediately
  before proceeding into the next bughunt window.

- If the second run fails, you must treat the previous frontier as the known
  baseline until the session lands a passing run that reproduces it.

- Treat every successful pass that improves frame frontier as a **ratchet
  milestone**: run pre-commit again with no new args as a post-fix proof before
  you commit. Use the final front (`last_checked_frame` in state) as the new
  session target in the same run.
- If `ZELDA3_PRECOMMIT_TARGET_FRAME` is used explicitly for a focused run, the
  gate must still update ratchet state upward on pass and you should immediately
  re-run the gate to confirm the next frontier is stable.

At the end of the session, spend a moment on a brief retrospective: what was
the slowest part of this investigation, and what one change (tool, doc, recipe,
checkpoint) would have halved it? If the answer is cheap, do it; if not, record
it in memory as a candidate improvement. Process improvements follow the same
cleanliness bar as fixes: committed only when finished, documented, and off by
default.

## Fast repro tools

- `python3 scripts/parity_probe.py --around <frame> [--window 40] [--capture]`
  re-runs one divergence window from the newest pre-commit `run-*/` inputs,
  resuming from a paired Rust+oracle checkpoint at `around-60` (saved on the
  first run; reused only while the parity binary is byte-identical). It prints
  the exact command, refuses a binary older than the Rust sources
  (`--allow-stale` overrides), and `--dry-run` stops there. `--capture` selects
  the instrumented core and summarizes `display_oracle.jsonl` for `around±3`:
  which domain diverges (registers/cgram/live+presented OAM/windows/mode7) plus
  decoded OAM slots. Never run two probes at once.
- `ZELDA3_PRECOMMIT_RESUME=1` makes the pre-commit gate keep a rolling paired
  checkpoint in `.git/precommit-snes9x-parity-checkpoint/` so each run replays
  only the new window instead of the whole frontier. Any rebuild (or route
  change) invalidates it and falls back to a full replay from frame 0; a failing
  run never advances the checkpoint. Unset, the gate behaves exactly as before.

## If you can't land a clean fix this session

Commit nothing half-done. Instead write up the diagnosis (divergence frame,
address, `whoowns` output, suspected class, what you ruled out) to persistent
memory so the next session starts where you stopped.
