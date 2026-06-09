#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
HOST="${STEAMDECK_HOST:-}"
TARBALL="${1:-target/steamdeck-linux-container/zelda3-steamdeck.tar.gz}"
CHECKSUM_FILE="${TARBALL}.sha256"
REMOTE_DIR="${STEAMDECK_REMOTE_DIR:-/home/deck/zelda3-rs-verify}"
LOCAL_LOG="${STEAMDECK_LOCAL_LOG:-$(dirname "$TARBALL")/steamdeck-verification.log}"
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
ssh "$HOST" "rm -rf '$REMOTE_DIR' && mkdir -p '$REMOTE_DIR'"
scp "$TARBALL" "$HOST:$REMOTE_DIR/zelda3-steamdeck.tar.gz"
scp "$CHECKSUM_FILE" "$HOST:$REMOTE_DIR/zelda3-steamdeck.tar.gz.sha256"
ssh "$HOST" "cd '$REMOTE_DIR' && sha256sum -c zelda3-steamdeck.tar.gz.sha256 && tar -xzf zelda3-steamdeck.tar.gz && cd zelda3-steamdeck && ./verify-on-deck.sh"
scp "$HOST:$REMOTE_DIR/zelda3-steamdeck/steamdeck-verification.log" "$LOCAL_LOG"
echo "copied Steam Deck verification log to $LOCAL_LOG"
