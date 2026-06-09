#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PROFILE="${PROFILE:-release}"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
DIST_DIR="${DIST_DIR:-dist}"
PACKAGE_NAME="${PACKAGE_NAME:-zelda3-steamdeck}"
PACKAGE_DIR="${DIST_DIR}/${PACKAGE_NAME}"
TARBALL="${DIST_DIR}/${PACKAGE_NAME}.tar.gz"
VERIFY_ONLY="${VERIFY_ONLY:-0}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 not found." >&2
    exit 2
  fi
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    echo "sha256sum or shasum not found." >&2
    exit 2
  fi
}

write_checksum_line() {
  local path="$1"
  printf '%s  %s\n' "$(sha256_file "$PACKAGE_DIR/$path")" "$path" >>"$PACKAGE_DIR/CHECKSUMS.sha256"
}

require_command tar
require_command mktemp
if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then
  echo "sha256sum or shasum not found." >&2
  exit 2
fi

if [[ "$VERIFY_ONLY" != "1" ]]; then
  if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" ]]; then
    echo "Steam Deck packaging must run on Linux x86_64. Use scripts/verify_steamdeck_linux_container.sh from macOS." >&2
    exit 2
  fi
  require_command cargo
  require_command python3
  require_command pkg-config
  pkg-config --exists libudev || { echo "libudev development metadata not found by pkg-config." >&2; exit 2; }
  pkg-config --exists alsa || { echo "ALSA development metadata not found by pkg-config." >&2; exit 2; }
  pkg-config --exists opus || { echo "Opus development metadata not found by pkg-config." >&2; exit 2; }
fi

if [[ "$PROFILE" == "release" ]]; then
  BINARY="${TARGET_DIR}/release/zelda3"
  BUILD_ARGS=(build -p zelda3-bin --release)
else
  BINARY="${TARGET_DIR}/debug/zelda3"
  BUILD_ARGS=(build -p zelda3-bin)
fi

if [[ "$VERIFY_ONLY" == "1" ]]; then
  TMP_DIR="$(mktemp -d)"
  trap 'rm -rf "$TMP_DIR"' EXIT
  BINARY="${TMP_DIR}/zelda3"
  printf '#!/usr/bin/env sh\necho "verify-only zelda3 stub"\n' >"$BINARY"
  chmod +x "$BINARY"
else
  cargo "${BUILD_ARGS[@]}"
  [[ -x "$BINARY" ]] || { echo "built binary not found or not executable: $BINARY" >&2; exit 1; }
  SMOKE_SAVE_DIR="$(mktemp -d)"
  trap 'rm -rf "$SMOKE_SAVE_DIR"' EXIT
  ZELDA3_SAVE_DIR="$SMOKE_SAVE_DIR" "$BINARY" --standalone-smoke 2
  ZELDA3_SAVE_DIR="$SMOKE_SAVE_DIR" "$BINARY" --sram-smoke
fi

rm -rf "$PACKAGE_DIR" "$TARBALL"
mkdir -p "$PACKAGE_DIR"
cp "$BINARY" "$PACKAGE_DIR/zelda3"

cat >"$PACKAGE_DIR/zelda3-rs.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128">
  <rect width="128" height="128" rx="20" fill="#1f2933"/>
  <path d="M64 14 24 106h80L64 14Z" fill="#d7b84f"/>
  <path d="M64 39 45 88h38L64 39Z" fill="#315f3f"/>
  <path d="M64 56 57 74h14L64 56Z" fill="#f4e08c"/>
</svg>
SVG

cat >"$PACKAGE_DIR/run-zelda3.sh" <<'RUNNER'
#!/usr/bin/env sh
set -eu
APP_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
export ZELDA3_STEAMDECK="${ZELDA3_STEAMDECK:-1}"
export ZELDA3_FULLSCREEN="${ZELDA3_FULLSCREEN:-1}"
export WGPU_BACKEND="${WGPU_BACKEND:-vulkan}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
export ZELDA3_SAVE_DIR="${ZELDA3_SAVE_DIR:-$DATA_HOME/zelda3-rs/saves}"
mkdir -p "$ZELDA3_SAVE_DIR"
cd "$APP_DIR"
exec ./zelda3 "$@"
RUNNER
chmod +x "$PACKAGE_DIR/run-zelda3.sh"

cat >"$PACKAGE_DIR/verify-on-deck.sh" <<'DECKVERIFY'
#!/usr/bin/env sh
set -eu
APP_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
LOG="$APP_DIR/steamdeck-verification.log"
FRAMES="${STEAMDECK_SMOKE_FRAMES:-2}"
run_logged() {
  printf '\n$ %s\n' "$*" | tee -a "$LOG"
  "$@" 2>&1 | tee -a "$LOG"
}
: >"$LOG"
{
  echo "zelda3-rs Steam Deck verification"
  date
  uname -a
  if [ -r /etc/os-release ]; then . /etc/os-release; echo "os=${PRETTY_NAME:-unknown}"; fi
  echo "DISPLAY=${DISPLAY:-}"
  echo "WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-}"
  echo "XDG_SESSION_TYPE=${XDG_SESSION_TYPE:-}"
  echo "Steam Deck hardware hint=$(test -r /sys/devices/virtual/dmi/id/product_name && cat /sys/devices/virtual/dmi/id/product_name || true)"
} | tee -a "$LOG"
if [ -f "$APP_DIR/CHECKSUMS.sha256" ]; then
  if command -v sha256sum >/dev/null 2>&1; then
    run_logged sh -c "cd '$APP_DIR' && sha256sum -c CHECKSUMS.sha256"
  elif command -v shasum >/dev/null 2>&1; then
    run_logged sh -c "cd '$APP_DIR' && shasum -a 256 -c CHECKSUMS.sha256"
  else
    echo "No checksum verifier found; skipping package integrity check." | tee -a "$LOG"
  fi
fi
SAVE_DIR="$(mktemp -d)"
trap 'rm -rf "$SAVE_DIR"' EXIT
run_logged env ZELDA3_SAVE_DIR="$SAVE_DIR" "$APP_DIR/zelda3" --standalone-smoke "$FRAMES"
run_logged env ZELDA3_SAVE_DIR="$SAVE_DIR" "$APP_DIR/zelda3" --sram-smoke
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
  echo "No graphical session detected; skipping frontend smoke." | tee -a "$LOG"
  echo "Rerun from Desktop Mode or Game Mode to verify frontend launch." | tee -a "$LOG"
else
  run_logged env ZELDA3_SAVE_DIR="$SAVE_DIR" ZELDA3_STEAMDECK="${ZELDA3_STEAMDECK:-1}" ZELDA3_FULLSCREEN="${ZELDA3_FULLSCREEN:-1}" WGPU_BACKEND="${WGPU_BACKEND:-vulkan}" "$APP_DIR/zelda3" --frontend-smoke "$FRAMES"
fi
echo "Steam Deck verification log written to $LOG"
DECKVERIFY
chmod +x "$PACKAGE_DIR/verify-on-deck.sh"

cat >"$PACKAGE_DIR/install-to-desktop-mode.sh" <<'INSTALLER'
#!/usr/bin/env sh
set -eu
SOURCE_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
APP_DIR="$DATA_HOME/zelda3-rs/app"
APPLICATIONS_DIR="$DATA_HOME/applications"
mkdir -p "$APP_DIR" "$APPLICATIONS_DIR"
cp "$SOURCE_DIR/zelda3" \
  "$SOURCE_DIR/run-zelda3.sh" \
  "$SOURCE_DIR/install-to-desktop-mode.sh" \
  "$SOURCE_DIR/verify-on-deck.sh" \
  "$SOURCE_DIR/zelda3-rs.desktop" \
  "$SOURCE_DIR/zelda3-rs.svg" \
  "$SOURCE_DIR/README.txt" \
  "$SOURCE_DIR/package-manifest.txt" \
  "$SOURCE_DIR/CHECKSUMS.sha256" \
  "$APP_DIR/"
chmod +x "$APP_DIR/zelda3" "$APP_DIR/run-zelda3.sh" "$APP_DIR/install-to-desktop-mode.sh" "$APP_DIR/verify-on-deck.sh"
cat >"$APPLICATIONS_DIR/zelda3-rs.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=zelda3-rs
Comment=The Legend of Zelda: A Link to the Past native Rust port
Exec=$APP_DIR/run-zelda3.sh
Path=$APP_DIR
Icon=$APP_DIR/zelda3-rs.svg
Terminal=false
Categories=Game;
StartupNotify=false
DESKTOP
chmod +x "$APPLICATIONS_DIR/zelda3-rs.desktop"
echo "Installed zelda3-rs desktop entry to $APPLICATIONS_DIR/zelda3-rs.desktop"
INSTALLER
chmod +x "$PACKAGE_DIR/install-to-desktop-mode.sh"

cat >"$PACKAGE_DIR/zelda3-rs.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=zelda3-rs
Comment=The Legend of Zelda: A Link to the Past native Rust port
Exec=sh -c 'cd "$(dirname "$1")" && exec ./run-zelda3.sh' sh %k
Icon=zelda3-rs
Terminal=false
Categories=Game;
StartupNotify=false
DESKTOP

cat >"$PACKAGE_DIR/README.txt" <<'README'
zelda3-rs Steam Deck package

Desktop Mode:
  1. Copy this folder to the Deck.
  2. Run ./install-to-desktop-mode.sh.
  3. Add zelda3-rs to Steam from the desktop entry.

Direct launch:
  ./run-zelda3.sh

On-Deck verification:
  ./verify-on-deck.sh

The wrapper enables Steam Deck defaults:
  ZELDA3_STEAMDECK=1
  ZELDA3_FULLSCREEN=1
  WGPU_BACKEND=vulkan
  ZELDA3_SAVE_DIR=${XDG_DATA_HOME:-$HOME/.local/share}/zelda3-rs/saves

The game assets are embedded in the executable. No ROM is needed at runtime.
README

GIT_COMMIT="$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
CREATED_UTC="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
cat >"$PACKAGE_DIR/package-manifest.txt" <<MANIFEST
package=$PACKAGE_NAME
created_utc=$CREATED_UTC
git_commit=$GIT_COMMIT
profile=$PROFILE
binary_sha256=$(sha256_file "$PACKAGE_DIR/zelda3")
runtime_assets=embedded
deck_defaults=ZELDA3_STEAMDECK,ZELDA3_FULLSCREEN,WGPU_BACKEND,ZELDA3_SAVE_DIR
MANIFEST

: >"$PACKAGE_DIR/CHECKSUMS.sha256"
for path in \
  zelda3 \
  run-zelda3.sh \
  install-to-desktop-mode.sh \
  verify-on-deck.sh \
  zelda3-rs.desktop \
  zelda3-rs.svg \
  README.txt \
  package-manifest.txt
do
  write_checksum_line "$path"
done

tar -C "$DIST_DIR" -czf "$TARBALL" "$PACKAGE_NAME"
scripts/verify_steamdeck_package.sh "$PACKAGE_DIR"
printf '%s  %s\n' "$(sha256_file "$TARBALL")" "$(basename "$TARBALL")" >"${TARBALL}.sha256"
echo "wrote $PACKAGE_DIR"
echo "wrote $TARBALL"
echo "wrote ${TARBALL}.sha256"
