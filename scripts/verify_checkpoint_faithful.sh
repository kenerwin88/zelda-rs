#!/usr/bin/env bash
# Verify checkpoint resume is byte-identical to a from-scratch run.
# Compares final-frame WRAM dumps AND the per-frame fingerprint record for the
# last frame (788-byte record incl. the audio leaf + wram pages 6/7).
set -uo pipefail

ROOT="/Users/missingno/Documents/zelda3-rs"
BIN="$ROOT/target/parity/zelda3"
ROM="$ROOT/saves/zelda3.sfc"
SAV="$ROOT/saves/zelda3-combined-route.sav"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

HACKS=(ZELDA3_SMV_SELECT_FILE_TIMING_HACKS=1 ZELDA3_SMV_LOADFILE_TIMING_HACKS=1 \
  ZELDA3_SMV_DUNGEON_TIMING_HACKS=1 ZELDA3_SMV_OVERWORLD_TIMING_HACKS=1 \
  ZELDA3_SMV_MESSAGING_TIMING_HACKS=1 ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS=1 \
  ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS=1)

# args: N N0
check_one() {
  local N="$1" N0="$2"
  echo "=== N=$N (checkpoint at N0=$N0) ==="

  # From-scratch to N
  env "${HACKS[@]}" ZELDA3_REPLAY_WRAM_DUMP="$TMP/fs.wram" \
    "$BIN" --replay-save "$ROM" "$SAV" "$N" --fingerprint-log "$TMP/fs.fp" >/dev/null 2>&1

  # Checkpoint at N0. Fingerprinting MUST be on here too: the per-frame audio
  # trace (zelda_render_audio_trace_dsp) advances the DSP as a side effect and
  # only runs when fingerprint/audio logging is enabled. zparity's checkpoint
  # seeding pass runs with fingerprinting on, so mirror that to capture the same
  # post-trace DSP state.
  env "${HACKS[@]}" "$BIN" --replay-save "$ROM" "$SAV" "$N0" \
    --save-state "$TMP/ck.sav" --fingerprint-log "$TMP/seed.fp" >/dev/null 2>&1

  # Resume from N0 to N
  env "${HACKS[@]}" ZELDA3_REPLAY_WRAM_DUMP="$TMP/rs.wram" \
    "$BIN" --replay-save "$ROM" "$SAV" "$N" \
    --load-state "$TMP/ck.sav" --fingerprint-log "$TMP/rs.fp" >/dev/null 2>&1

  # Compare WRAM
  if cmp -s "$TMP/fs.wram" "$TMP/rs.wram"; then
    echo "  WRAM: IDENTICAL"
  else
    echo "  WRAM: DIFFER"
    cmp -l "$TMP/fs.wram" "$TMP/rs.wram" 2>/dev/null | head -20
  fi

  # Compare fingerprint record for the last frame (N-1).
  # fs.fp record idx = N-1 ; rs.fp record idx = N-1-N0
  python3 - "$TMP/fs.fp" "$TMP/rs.fp" "$N" "$N0" <<'PY'
import sys
fs, rs, N, N0 = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
data_fs = open(fs,'rb').read()
data_rs = open(rs,'rb').read()
# Determine record size from total/count: fs has N records.
if len(data_fs) % N == 0:
    rec = len(data_fs)//N
else:
    rec = 788
fi = (N-1)
ri = (N-1-N0)
a = data_fs[fi*rec:(fi+1)*rec]
b = data_rs[ri*rec:(ri+1)*rec]
if a == b:
    print(f"  FINGERPRINT[{N-1}]: IDENTICAL (rec={rec})")
else:
    print(f"  FINGERPRINT[{N-1}]: DIFFER (rec={rec})")
    diffs = [i for i in range(min(len(a),len(b))) if a[i]!=b[i]]
    print("    first diff offsets:", diffs[:24])
PY
}

for spec in "4167:4166" "4250:4166" "30100:30000"; do
  check_one "${spec%%:*}" "${spec##*:}"
done
