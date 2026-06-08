#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit

echo "Configured git hooks from .githooks/"
echo "Pre-commit expects the local C checkout and ROM/parity fixtures documented in README.md."

