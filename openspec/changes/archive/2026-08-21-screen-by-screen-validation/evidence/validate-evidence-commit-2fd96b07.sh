#!/usr/bin/env bash
set -euo pipefail

evidence="2fd96b07e396fe1e988232864a9eefef824b3aa3"
bundle_path="docs/certifications/product-screens/f8e203b6/manifest.json"
resolved_evidence="$(git rev-parse "$evidence")"
parent="$(git rev-parse "$evidence^")"

echo "EVIDENCE_COMMIT=$resolved_evidence"
echo "EVIDENCE_PARENT=$parent"

git cat-file -e "$resolved_evidence:$bundle_path"
echo "EVIDENCE_CONTAINS_BUNDLE=PASS"

stderr_file="$(mktemp)"
trap 'rm -f "$stderr_file"' EXIT
set +e
git cat-file -e "$parent:$bundle_path" 2>"$stderr_file"
status=$?
set -e
echo "SOURCE_CONTAINS_BUNDLE_EXIT=$status"
test "$status" -ne 0
echo "SOURCE_BUNDLE_ABSENCE_CONTROL=PASS"
printf 'SOURCE_BUNDLE_ABSENCE_STDERR='
tr '\n' ' ' < "$stderr_file"
printf '\n'
