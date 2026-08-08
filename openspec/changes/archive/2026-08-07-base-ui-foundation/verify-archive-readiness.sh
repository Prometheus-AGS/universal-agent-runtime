#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

implementation_commit="e92670e248d7e02fed764edc16a7fabcf9d84dca"
test "$(git cat-file -t "$implementation_commit")" = "commit"
test "$(git show -s --format=%s "$implementation_commit")" = \
  "feat: swap radix-ui for @base-ui/react and regenerate shadcn components"

node - <<'NODE'
const packageJson = require("./frontend/package.json");
const components = require("./frontend/components.json");

if (packageJson.dependencies["@base-ui/react"] !== "1.6.0") {
  throw new Error("@base-ui/react must be pinned to 1.6.0");
}
if (components.style !== "base-vega") {
  throw new Error("components.json must select base-vega");
}
if (components.tailwind.baseColor !== "neutral") {
  throw new Error("components.json must select the neutral base color");
}
NODE

base_ui_wrappers="$(rg -l '@base-ui/react' frontend/src/components/ui | wc -l | tr -d ' ')"
test "$base_ui_wrappers" -ge 1

if rg -l '@radix-ui/' frontend/src --glob '!**/ui-radix-backup/**' >/dev/null; then
  echo "production frontend source still imports a Radix UI package" >&2
  exit 1
fi

test "$(awk '/^- \[ \]/{count++} END{print count+0}' \
  openspec/changes/base-ui-foundation/tasks.md)" -eq 0

openspec validate base-ui-foundation --strict
openspec status --change base-ui-foundation --json | jq -e \
  '.isComplete == true
   and ([.artifacts[].id] | sort) == ["design", "proposal", "specs", "tasks"]
   and ([.artifacts[].status] | all(. == "done"))' >/dev/null

printf 'base-ui-foundation archive readiness: PASS (%s Base UI wrappers)\n' \
  "$base_ui_wrappers"
