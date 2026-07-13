#!/usr/bin/env bash
# Compares a freshly generated lcov report against the recorded baseline in
# docs/coverage-baseline.md and prints the per-file delta. Fails if any file
# drops more than 5 points vs. the baseline. Baseline percentages are parsed
# from the "| path | pct |" table rows in docs/coverage-baseline.md.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

lcov_file="${1:-lcov.info}"
baseline_doc="docs/coverage-baseline.md"
drift_threshold=5

if [[ ! -f "$lcov_file" ]]; then
  echo "coverage-drift.sh: lcov file not found: $lcov_file" >&2
  exit 1
fi

if [[ ! -f "$baseline_doc" ]]; then
  echo "coverage-drift.sh: no baseline recorded yet at $baseline_doc — skipping drift check." >&2
  exit 0
fi

# Compute current per-file line coverage percentage from the lcov file:
# SF:<path> starts a record; LF/LH give total/hit lines until end_of_record.
awk -v threshold="$drift_threshold" -v baseline_doc="$baseline_doc" '
  BEGIN {
    while ((getline line < baseline_doc) > 0) {
      if (line ~ /^\| `?[^|]+`? \| *[0-9.]+% *\|/) {
        gsub(/`/, "", line)
        split(line, cols, "|")
        gsub(/^ +| +$/, "", cols[2])
        gsub(/^ +| +$| %/, "", cols[3])
        baseline[cols[2]] = cols[3]
      }
    }
    close(baseline_doc)
    fail = 0
  }
  /^SF:/ { file = substr($0, 4); lf = 0; lh = 0 }
  /^LF:/ { lf = substr($0, 4) }
  /^LH:/ { lh = substr($0, 4) }
  /^end_of_record/ {
    if (lf > 0) {
      pct = (lh / lf) * 100
      if (file in baseline) {
        delta = pct - baseline[file]
        printf "%-60s current=%.1f%% baseline=%.1f%% delta=%+.1f%%\n", file, pct, baseline[file], delta
        if (delta < -threshold) {
          printf "  DROP exceeds %d points: %s\n", threshold, file
          fail = 1
        }
      } else {
        printf "%-60s current=%.1f%% (no baseline entry)\n", file, pct
      }
    }
  }
  END { exit fail }
' "$lcov_file"
