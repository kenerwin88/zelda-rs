#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 6 ]]; then
  echo "usage: $0 CACHE_DIR ROM_PATH PAIRED_CHECKPOINT OUTPUT_DIR FRAMES_FROM_CHECKPOINT TRIAL_START_HOST [--from-host REPORT_START_HOST] [REPLAY_OPTION...]" >&2
  exit 2
fi

cache_dir=$1
rom_path=$2
checkpoint=$3
output_dir=$4
frames=$5
trial_start=$6
shift 6

report_start=$trial_start
if [[ ${1:-} == --from-host ]]; then
  if [[ $# -lt 2 ]]; then
    echo "--from-host requires a host number" >&2
    exit 2
  fi
  report_start=$2
  shift 2
fi

mkdir -p "$output_dir"
first_log="$output_dir/first.log"
watched_log="$output_dir/watched.log"

scripts/native_exact_cpu_diagnose.sh \
  "$cache_dir" "$rom_path" "$checkpoint" "$output_dir/first" \
  "$frames" "$trial_start" "$@" > "$first_log" 2>&1

watch_offset=$(python3 scripts/native_exact_cpu_report.py \
  "$first_log" --from-host "$report_start" --address-only)
if [[ -z $watch_offset ]]; then
  python3 scripts/native_exact_cpu_report.py "$first_log" --from-host "$report_start"
  exit 0
fi

ZELDA3_NATIVE_EXACT_CPU_WATCH_WRAM_ADDR="$watch_offset" \
  scripts/native_exact_cpu_diagnose.sh \
  "$cache_dir" "$rom_path" "$checkpoint" "$output_dir/watched" \
  "$frames" "$trial_start" "$@" > "$watched_log" 2>&1
python3 scripts/native_exact_cpu_report.py "$watched_log" --from-host "$report_start"
