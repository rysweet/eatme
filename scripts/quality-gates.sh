#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export TMPDIR="${TMPDIR:-$ROOT/.cargo-tmp}"
mkdir -p "$TMPDIR"

MODULE_MAX_LINES="${MODULE_MAX_LINES:-700}"
COVERAGE_FAIL_UNDER="${COVERAGE_FAIL_UNDER:-70}"

echo "==> cargo fmt"
cargo fmt --check

echo "==> cargo clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "==> cargo test"
cargo test --workspace --all-features

echo "==> module size (<= ${MODULE_MAX_LINES} lines)"
find crates -name '*.rs' \
  -not -path '*/target/*' \
  -not -path '*/tests/*' \
  -not -name 'tests.rs' \
  -not -name '*_tests.rs' \
  -exec wc -l {} + \
  | awk -v max="$MODULE_MAX_LINES" '$2 != "total" && $1 > max { print; bad=1 } END { exit bad }'

echo "==> cargo llvm-cov (${COVERAGE_FAIL_UNDER}% line coverage)"
cargo llvm-cov --workspace --all-features --fail-under-lines "$COVERAGE_FAIL_UNDER" --summary-only
