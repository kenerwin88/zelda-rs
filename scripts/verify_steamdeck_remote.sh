#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
HOST="${STEAMDECK_HOST:-}"
TARBALL="${1:-target/steamdeck-linux-container/zelda3-steamdeck.tar.gz}"
CHECKSUM_FILE="${TARBALL}.sha256"
REMOTE_DIR="${STEAMDECK_REMOTE_DIR:-/home/deck/zelda3-rs-verify}"
LOCAL_LOG="${STEAMDECK_LOCAL_LOG:-$(dirname "$TARBALL")/steamdeck-verification.log}"
SSH_OPTS=()
if [[ -n "${STEAMDECK_SSH_OPTS:-}" ]]; then
  # shellcheck disable=SC2206
  SSH_OPTS=(${STEAMDECK_SSH_OPTS})
fi
if [[ -z "$HOST" ]]; then echo "Set STEAMDECK_HOST to an ssh target, for example deck@steamdeck." >&2; exit 2; fi
if [[ ! -f "$TARBALL" ]]; then
  echo "Package tarball not found: $TARBALL" >&2
  echo "Build it first with scripts/verify_steamdeck_linux_container.sh or scripts/package_steamdeck.sh." >&2
  exit 2
fi
if [[ ! -f "$CHECKSUM_FILE" ]]; then
  echo "Package checksum not found: $CHECKSUM_FILE" >&2
  echo "Build it first with scripts/verify_steamdeck_linux_container.sh or scripts/package_steamdeck.sh." >&2
  exit 2
fi
ssh "${SSH_OPTS[@]}" "$HOST" "rm -rf '$REMOTE_DIR' && mkdir -p '$REMOTE_DIR'"
scp "${SSH_OPTS[@]}" "$TARBALL" "$HOST:$REMOTE_DIR/zelda3-steamdeck.tar.gz"
scp "${SSH_OPTS[@]}" "$CHECKSUM_FILE" "$HOST:$REMOTE_DIR/zelda3-steamdeck.tar.gz.sha256"
ssh "${SSH_OPTS[@]}" "$HOST" "cd '$REMOTE_DIR' && if command -v sha256sum >/dev/null 2>&1; then sha256sum -c zelda3-steamdeck.tar.gz.sha256; else shasum -a 256 -c zelda3-steamdeck.tar.gz.sha256; fi && tar -xzf zelda3-steamdeck.tar.gz && cd zelda3-steamdeck && ./verify-on-deck.sh"
scp "${SSH_OPTS[@]}" "$HOST:$REMOTE_DIR/zelda3-steamdeck/steamdeck-verification.log" "$LOCAL_LOG"
echo "copied Steam Deck verification log to $LOCAL_LOG"
