#!/usr/bin/env bash
set -euo pipefail
PACKAGE_DIR="${1:-dist/zelda3-steamdeck}"
fail() { echo "verify_steamdeck_package: $*" >&2; exit 1; }
[[ -d "$PACKAGE_DIR" ]] || fail "package directory not found: $PACKAGE_DIR"
for path in zelda3 run-zelda3.sh install-to-desktop-mode.sh verify-on-deck.sh zelda3-rs.desktop zelda3-rs.svg README.txt; do
  [[ -e "$PACKAGE_DIR/$path" ]] || fail "missing $path"
done
[[ -x "$PACKAGE_DIR/zelda3" ]] || fail "zelda3 is not executable"
[[ -x "$PACKAGE_DIR/run-zelda3.sh" ]] || fail "run-zelda3.sh is not executable"
[[ -x "$PACKAGE_DIR/install-to-desktop-mode.sh" ]] || fail "install-to-desktop-mode.sh is not executable"
[[ -x "$PACKAGE_DIR/verify-on-deck.sh" ]] || fail "verify-on-deck.sh is not executable"
bash -n "$PACKAGE_DIR/run-zelda3.sh"
bash -n "$PACKAGE_DIR/install-to-desktop-mode.sh"
bash -n "$PACKAGE_DIR/verify-on-deck.sh"
grep -q 'ZELDA3_STEAMDECK' "$PACKAGE_DIR/run-zelda3.sh" || fail "wrapper does not set ZELDA3_STEAMDECK"
grep -q 'ZELDA3_FULLSCREEN' "$PACKAGE_DIR/run-zelda3.sh" || fail "wrapper does not set ZELDA3_FULLSCREEN"
grep -q 'WGPU_BACKEND' "$PACKAGE_DIR/run-zelda3.sh" || fail "wrapper does not set WGPU_BACKEND"
grep -q 'ZELDA3_SAVE_DIR' "$PACKAGE_DIR/run-zelda3.sh" || fail "wrapper does not set ZELDA3_SAVE_DIR"
grep -q 'dirname "$1"' "$PACKAGE_DIR/zelda3-rs.desktop" || fail "desktop entry must launch relative to package folder"
grep -q '^Categories=Game;' "$PACKAGE_DIR/zelda3-rs.desktop" || fail "desktop entry missing Game category"
if command -v desktop-file-validate >/dev/null 2>&1; then desktop-file-validate "$PACKAGE_DIR/zelda3-rs.desktop"; fi
if [[ "$(uname -s)" == "Linux" ]]; then
  SMOKE_SAVE_DIR="$(mktemp -d)"
  trap 'rm -rf "$SMOKE_SAVE_DIR"' EXIT
  ZELDA3_SAVE_DIR="$SMOKE_SAVE_DIR" "$PACKAGE_DIR/zelda3" --standalone-smoke 2 >/dev/null
  ZELDA3_SAVE_DIR="$SMOKE_SAVE_DIR" "$PACKAGE_DIR/zelda3" --sram-smoke >/dev/null
  if [[ "${STEAMDECK_FRONTEND_SMOKE:-0}" == "1" ]]; then
    ZELDA3_SAVE_DIR="$SMOKE_SAVE_DIR" "$PACKAGE_DIR/zelda3" --frontend-smoke 2 >/dev/null
  fi
  if command -v ldd >/dev/null 2>&1; then
    ldd "$PACKAGE_DIR/zelda3" >"$PACKAGE_DIR/ldd.txt"
    if grep -q 'not found' "$PACKAGE_DIR/ldd.txt"; then cat "$PACKAGE_DIR/ldd.txt" >&2; fail "packaged binary has unresolved shared libraries"; fi
  fi
fi
echo "verify_steamdeck_package: ok"
