#!/usr/bin/env bash
# Detect available orchestration frameworks in the consuming workspace.
# Usage: from repository root — bash prometheus-entity-skills/entity-graph-setup/scripts/detect-orchestrators.sh
#        or from this directory with cwd set to workspace root.

set -euo pipefail

check_dir_or_file() {
  if [[ -e "$1" ]]; then
    echo "true"
  else
    echo "false"
  fi
}

KBD=$(check_dir_or_file ".kbd-orchestrator/project.json")
EVOLVER=$(check_dir_or_file ".evolver")
REFINER=$(check_dir_or_file ".refiner")
OPENSPEC=$(check_dir_or_file "openspec")

printf '{"kbd":{"available":%s},"evolver":{"available":%s},"refiner":{"available":%s},"openspec":{"available":%s}}\n' \
  "$KBD" "$EVOLVER" "$REFINER" "$OPENSPEC"
