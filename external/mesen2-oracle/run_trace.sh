#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../.." && pwd)

rom=${1:-/Users/missingno/Documents/zelda3/zelda3.sfc}
frames=${2:-180}
output=${3:-$script_dir/startup-apui-dsp.jsonl}
mesen=${MESEN2_BIN:-$script_dir/local/Mesen.app/Contents/MacOS/Mesen}

case "$output" in
  /*) ;;
  *) output="$repo_root/$output" ;;
esac

if [ ! -x "$mesen" ]; then
  echo "Mesen2 binary not found or not executable: $mesen" >&2
  echo "Set MESEN2_BIN=/path/to/Mesen or place Mesen.app under external/mesen2-oracle/local/." >&2
  exit 2
fi

mkdir -p "$(dirname -- "$output")"

M2_TRACE_OUT="$output" \
M2_TRACE_FRAMES="$frames" \
M2_TRACE_PIXEL_X="${M2_TRACE_PIXEL_X:-}" \
M2_TRACE_PIXEL_Y="${M2_TRACE_PIXEL_Y:-}" \
M2_TRACE_CGRAM_START="${M2_TRACE_CGRAM_START:-}" \
M2_TRACE_CGRAM_COUNT="${M2_TRACE_CGRAM_COUNT:-}" \
M2_TRACE_SPC_VARS="${M2_TRACE_SPC_VARS:-}" \
"$mesen" \
  --testRunner \
  --enableStdout \
  --doNotSaveSettings \
  --timeout="$frames" \
  --debug.scriptWindow.allowIoOsAccess=true \
  "$script_dir/trace_apui_dsp.lua" \
  "$rom"

echo "wrote $output"
echo "repo root: $repo_root"
