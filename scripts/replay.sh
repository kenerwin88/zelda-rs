#!/usr/bin/env bash
# Run a parity replay without ever referencing the C repo.
#
# Reference is Rust-vs-Rust only: this repo + zelda3-rs-old (perfect parity). The ROM
# defaults to saves/zelda3.sfc (gitignored), NEVER ../zelda3/zelda3.sfc. All 7 SMV timing
# hacks are set automatically.
#
#   scripts/replay.sh <frames>                       # this repo (target/parity)
#   scripts/replay.sh <frames> old                   # the zelda3-rs-old clone
#   ZELDA3_REPLAY_WRAM_DUMP=/tmp/x.bin scripts/replay.sh 8857
#   ZELDA3_ASSERT_NATIVE_COHERENT=1 ZELDA3_ASSERT_COHERENT_FRAME=8857 scripts/replay.sh 8858
#
# Any ZELDA3_* env (dumps, traces, coherence checker) is inherited.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OLD_REPO="${ZELDA3_OLD_REPO:-$HOME/Documents/zelda3-rs-old}"
ROM="${ZELDA3_ROM:-$ROOT/saves/zelda3.sfc}"
SAVE="${ZELDA3_REPLAY_SAVE:-$ROOT/saves/zelda3-combined-route.sav}"

frames="${1:?usage: replay.sh <frames> [old]}"
which="${2:-new}"
if [ "$which" = "old" ]; then
  BIN="$OLD_REPO/target/release/zelda3"
else
  BIN="$ROOT/target/parity/zelda3"
fi

[ -f "$ROM" ] || { echo "ROM not found at $ROM (copy your .sfc there; do NOT use ../zelda3)" >&2; exit 1; }
[ -x "$BIN" ] || { echo "binary not found: $BIN (build it first)" >&2; exit 1; }

export ZELDA3_SMV_SELECT_FILE_TIMING_HACKS=1 \
       ZELDA3_SMV_LOADFILE_TIMING_HACKS=1 \
       ZELDA3_SMV_DUNGEON_TIMING_HACKS=1 \
       ZELDA3_SMV_OVERWORLD_TIMING_HACKS=1 \
       ZELDA3_SMV_MESSAGING_TIMING_HACKS=1 \
       ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS=1 \
       ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS=1

exec "$BIN" --replay-save "$ROM" "$SAVE" "$frames"
