#!/usr/bin/env bash
# Summarize a cargo-mutants report directory.
# Usage: mutation-summarize.sh <report-dir>

set -euo pipefail

REPORT_DIR="${1:-}"
if [ -z "$REPORT_DIR" ]; then
    echo "Usage: $0 <report-dir>" >&2
    exit 1
fi

if [ ! -d "$REPORT_DIR" ]; then
    echo "No mutation report found at $REPORT_DIR" >&2
    exit 1
fi

SUMMARY_FILE="$REPORT_DIR/summary.txt"
MUTANTS_FILE="$REPORT_DIR/mutants.txt"

if [ -f "$SUMMARY_FILE" ]; then
    echo "=== Mutation Summary ==="
    cat "$SUMMARY_FILE"
fi

if [ -f "$MUTANTS_FILE" ]; then
    TOTAL=$(grep -c '' "$MUTANTS_FILE" || true)
    CAUGHT=$(grep -c 'caught' "$MUTANTS_FILE" || true)
    MISSED=$(grep -c 'missed' "$MUTANTS_FILE" || true)
    TIMEOUT=$(grep -c 'timeout' "$MUTANTS_FILE" || true)
    UNCHECKED=$(grep -c 'unchecked' "$MUTANTS_FILE" || true)
    echo "Total mutants: $TOTAL"
    echo "Caught: $CAUGHT"
    echo "Missed: $MISSED"
    echo "Timeout: $TIMEOUT"
    echo "Unchecked: $UNCHECKED"
fi

echo "Report directory: $REPORT_DIR"
