#!/usr/bin/env bash
# Gate 4: shard invariance. After clearing the checkpoint cache, `check` with
# 1 shard and 12 shards must produce the SAME verdict line (both MATCH up to the
# frame count, or the same DIVERGE frame). This proves checkpoint-boundary
# false-positives are gone.
set -uo pipefail
cd /Users/missingno/Documents/zelda3-rs
FRAMES="${1:-50000}"
ZP=target/debug/zparity

rm -rf .cache/parity-golden/ck
echo "--- shards 1 ---"
S1=$("$ZP" check --frames "$FRAMES" --shards 1 2>&1 | grep -E "^(MATCH|DIVERGE)")
echo "$S1"
rm -rf .cache/parity-golden/ck
echo "--- shards 12 ---"
S12=$("$ZP" check --frames "$FRAMES" --shards 12 2>&1 | grep -E "^(MATCH|DIVERGE)")
echo "$S12"

# Compare the verdict, ignoring the timing/shards columns.
norm() { echo "$1" | sed -E 's/[0-9]+ shards//; s/, [0-9.]+s//'; }
if [ "$(norm "$S1")" = "$(norm "$S12")" ]; then
  echo "SHARD-INVARIANT: PASS"
else
  echo "SHARD-INVARIANT: FAIL"
fi
