#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
IMAGE="${IMAGE:-rust:1-bookworm}"
PLATFORM="${PLATFORM:-linux/amd64}"
DIST_DIR="${DIST_DIR:-target/steamdeck-linux-container}"
PROFILE="${PROFILE:-debug}"
command -v docker >/dev/null 2>&1 || { echo "docker not found." >&2; exit 2; }
docker info >/dev/null
docker run --rm \
  --platform "$PLATFORM" \
  -v "$ROOT:/work" \
  -w /work \
  -e CARGO_TARGET_DIR=/work/target/linux-container \
  -e PROFILE="$PROFILE" \
  -e DIST_DIR="$DIST_DIR" \
  "$IMAGE" \
  bash -lc 'set -euo pipefail; export PATH="/usr/local/cargo/bin:$PATH"; apt-get update; apt-get install -y --no-install-recommends pkg-config libudev-dev libasound2-dev libopus-dev ca-certificates; scripts/package_steamdeck.sh'
