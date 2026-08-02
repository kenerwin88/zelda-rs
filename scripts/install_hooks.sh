#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit

echo "Configured git hooks from .githooks/"
echo "Pre-commit expects parity fixtures and ROM setup documented in README.md."
