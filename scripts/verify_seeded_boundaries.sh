#!/usr/bin/env bash
# Fast direct check of checkpoint faithfulness using the SAME seeding pass zparity
# uses (one --save-state-at run writing all boundary checkpoints), then resumes
# from each checkpoint by one frame and compares WRAM to a continuous run. This
# reproduces the cumulative-corruption path without the full shard rollup compare.
set -uo pipefail
cd /Users/missingno/Documents/zelda3-rs
BIN=target/parity/zelda3; ROM=saves/zelda3.sfc; SAV=saves/zelda3-combined-route.sav
END="${1:-30000}"; STEP="${2:-2500}"
HACKS=(ZELDA3_SMV_SELECT_FILE_TIMING_HACKS=1 ZELDA3_SMV_LOADFILE_TIMING_HACKS=1 \
  ZELDA3_SMV_DUNGEON_TIMING_HACKS=1 ZELDA3_SMV_OVERWORLD_TIMING_HACKS=1 \
  ZELDA3_SMV_MESSAGING_TIMING_HACKS=1 ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS=1 \
  ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS=1)
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

# Seed all boundary checkpoints in ONE pass (exactly like zparity).
sa=()
for ((f=STEP; f<END; f+=STEP)); do sa+=(--save-state-at "$f:$TMP/ck_$f.sav"); done
env "${HACKS[@]}" "$BIN" --replay-save "$ROM" "$SAV" "$END" "${sa[@]}" \
  --fingerprint-log "$TMP/seed.fp" >/dev/null 2>&1

fail=0
for ((f=STEP; f<END; f+=STEP)); do
  n=$((f+1))
  env "${HACKS[@]}" ZELDA3_REPLAY_WRAM_DUMP="$TMP/cont_$n.bin" \
    "$BIN" --replay-save "$ROM" "$SAV" "$n" --fingerprint-log "$TMP/c.fp" >/dev/null 2>&1
  env "${HACKS[@]}" ZELDA3_REPLAY_WRAM_DUMP="$TMP/res_$n.bin" \
    "$BIN" --replay-save "$ROM" "$SAV" "$n" --load-state "$TMP/ck_$f.sav" >/dev/null 2>&1
  if cmp -s "$TMP/cont_$n.bin" "$TMP/res_$n.bin"; then
    echo "boundary $f -> resume@$n WRAM IDENTICAL"
  else
    nd=$(cmp -l "$TMP/cont_$n.bin" "$TMP/res_$n.bin" 2>/dev/null | wc -l | tr -d ' ')
    echo "boundary $f -> resume@$n WRAM DIFFER ($nd bytes)"
    fail=1
  fi
done
echo "OVERALL: $([ $fail -eq 0 ] && echo PASS || echo FAIL)"
