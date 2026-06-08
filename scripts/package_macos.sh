#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PROFILE="${PROFILE:-release}"
IDENTITY="${SIGN_IDENTITY:--}"
BUNDLE_ID="${BUNDLE_ID:-com.zelda3rs.zelda3}"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
ARCH="$(uname -m)"
DIST_DIR="${DIST_DIR:-dist}"
PACKAGE_NAME="${PACKAGE_NAME:-zelda3-macos-${ARCH}}"
PACKAGE_DIR="${DIST_DIR}/${PACKAGE_NAME}"
ZIP_PATH="${DIST_DIR}/${PACKAGE_NAME}.zip"
NOTARY_PROFILE="${NOTARY_PROFILE:-}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS packaging requires macOS because it uses codesign." >&2
  exit 2
fi

if ! command -v codesign >/dev/null 2>&1; then
  echo "codesign not found." >&2
  exit 2
fi

if [[ "$PROFILE" == "release" ]]; then
  cargo build -p zelda3-bin --release
  BINARY="${TARGET_DIR}/release/zelda3"
else
  cargo build -p zelda3-bin
  BINARY="${TARGET_DIR}/debug/zelda3"
fi

if [[ ! -x "$BINARY" ]]; then
  echo "built binary not found or not executable: $BINARY" >&2
  exit 1
fi

rm -rf "$PACKAGE_DIR" "$ZIP_PATH"
mkdir -p "$PACKAGE_DIR"
cp "$BINARY" "$PACKAGE_DIR/zelda3"

SIGN_ARGS=(
  --force
  --sign "$IDENTITY"
  --identifier "$BUNDLE_ID"
  --options runtime
)

if [[ "$IDENTITY" != "-" ]]; then
  SIGN_ARGS+=(--timestamp)
fi

codesign "${SIGN_ARGS[@]}" "$PACKAGE_DIR/zelda3"
codesign --verify --strict --verbose=2 "$PACKAGE_DIR/zelda3"

ditto -c -k --keepParent "$PACKAGE_DIR" "$ZIP_PATH"

if [[ -n "$NOTARY_PROFILE" ]]; then
  if [[ "$IDENTITY" == "-" ]]; then
    echo "NOTARY_PROFILE requires a real Developer ID Application signing identity." >&2
    exit 2
  fi
  xcrun notarytool submit "$ZIP_PATH" --keychain-profile "$NOTARY_PROFILE" --wait
fi

echo "wrote $PACKAGE_DIR"
echo "wrote $ZIP_PATH"
codesign -dv "$PACKAGE_DIR/zelda3" 2>&1 | sed 's/^/codesign: /'
