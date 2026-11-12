#!/usr/bin/env bash
# The compile gate before a commit: formats checked, analyzers and linters
# clean, everything compiled. No tests run here. Kept light on the machine:
# low priority, two jobs, one tool after the other.
#
# Usage: tool/gate.sh [rust] [dart]   (both when none is named)
set -euo pipefail
cd "$(dirname "$0")/.."
want="${*:-rust dart}"

if [[ $want == *rust* ]]; then
  pkill -x rust-analyzer 2>/dev/null || true
  (cd desktop &&
    nice -n 19 env CARGO_BUILD_JOBS=2 cargo fmt --check &&
    nice -n 19 env CARGO_BUILD_JOBS=2 cargo clippy -j 2 --workspace --all-targets -- -D warnings)
fi
if [[ $want == *dart* ]]; then
  nice -n 19 dart format --output=none --set-exit-if-changed lib test
  nice -n 19 flutter analyze
fi
