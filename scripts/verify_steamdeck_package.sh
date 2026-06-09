#!/usr/bin/env bash
set -euo pipefail
PACKAGE_DIR="${1:-dist/zelda3-steamdeck}"
fail() { echo "verify_steamdeck_package: $*" >&2; exit 1; }
[[ -d "$PACKAGE_DIR" ]] || fail "package directory not found: $PACKAGE_DIR"
for path in zelda3 run-zelda3.sh install-to-desktop-mode.sh verify-on-deck.sh zelda3-rs.desktop zelda3-rs.svg README.txt package-manifest.txt CHECKSUMS.sha256; do
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
grep -q 'detect_graphical_session' "$PACKAGE_DIR/verify-on-deck.sh" || fail "on-Deck verifier does not discover graphical sessions"
grep -q 'WAYLAND_DISPLAY' "$PACKAGE_DIR/verify-on-deck.sh" || fail "on-Deck verifier does not handle Wayland"
grep -q 'dirname "$1"' "$PACKAGE_DIR/zelda3-rs.desktop" || fail "desktop entry must launch relative to package folder"
grep -q '^Categories=Game;' "$PACKAGE_DIR/zelda3-rs.desktop" || fail "desktop entry missing Game category"
grep -q '^runtime_assets=embedded$' "$PACKAGE_DIR/package-manifest.txt" || fail "manifest does not declare embedded runtime assets"
grep -q '^binary_sha256=' "$PACKAGE_DIR/package-manifest.txt" || fail "manifest missing binary checksum"
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$PACKAGE_DIR" && sha256sum -c CHECKSUMS.sha256 >/dev/null)
elif command -v shasum >/dev/null 2>&1; then
  (cd "$PACKAGE_DIR" && shasum -a 256 -c CHECKSUMS.sha256 >/dev/null)
else
  fail "sha256sum or shasum not found"
fi
if command -v desktop-file-validate >/dev/null 2>&1; then desktop-file-validate "$PACKAGE_DIR/zelda3-rs.desktop"; fi
INSTALL_TEST_DIR="$(mktemp -d)"
XDG_DATA_HOME="$INSTALL_TEST_DIR/data" HOME="$INSTALL_TEST_DIR/home" "$PACKAGE_DIR/install-to-desktop-mode.sh" >/dev/null
INSTALLED_APP_DIR="$INSTALL_TEST_DIR/data/zelda3-rs/app"
INSTALLED_DESKTOP="$INSTALL_TEST_DIR/data/applications/zelda3-rs.desktop"
[[ -x "$INSTALLED_APP_DIR/zelda3" ]] || fail "installer did not copy executable binary"
[[ -x "$INSTALLED_APP_DIR/run-zelda3.sh" ]] || fail "installer did not copy executable wrapper"
[[ -x "$INSTALLED_APP_DIR/verify-on-deck.sh" ]] || fail "installer did not copy executable verifier"
[[ -f "$INSTALLED_APP_DIR/CHECKSUMS.sha256" ]] || fail "installer did not copy checksums"
[[ -f "$INSTALLED_DESKTOP" ]] || fail "installer did not write desktop entry"
grep -q "^Exec=$INSTALLED_APP_DIR/run-zelda3.sh$" "$INSTALLED_DESKTOP" || fail "installed desktop entry has wrong Exec"
grep -q "^Path=$INSTALLED_APP_DIR$" "$INSTALLED_DESKTOP" || fail "installed desktop entry has wrong Path"
grep -q "^Icon=$INSTALLED_APP_DIR/zelda3-rs.svg$" "$INSTALLED_DESKTOP" || fail "installed desktop entry has wrong Icon"
rm -rf "$INSTALL_TEST_DIR"
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
