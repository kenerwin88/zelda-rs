#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
HOST="${STEAMDECK_HOST:-}"
TARBALL="${1:-target/steamdeck-linux-container/zelda3-steamdeck.tar.gz}"
REMOTE_DIR="${STEAMDECK_REMOTE_DIR:-/home/deck/zelda3-rs-verify}"
if [[ -z "$HOST" ]]; then echo "Set STEAMDECK_HOST to an ssh target, for example deck@steamdeck." >&2; exit 2; fi
if [[ ! -f "$TARBALL" ]]; then
  echo "Package tarball not found: $TARBALL" >&2
  echo "Build it first with scripts/verify_steamdeck_linux_container.sh or scripts/package_steamdeck.sh." >&2
  exit 2
fi
ssh "$HOST" "rm -rf '$REMOTE_DIR' && mkdir -p '$REMOTE_DIR'"
scp "$TARBALL" "$HOST:$REMOTE_DIR/zelda3-steamdeck.tar.gz"
ssh "$HOST" "cd '$REMOTE_DIR' && tar -xzf zelda3-steamdeck.tar.gz && cd zelda3-steamdeck && ./verify-on-deck.sh"
