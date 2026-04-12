#!/usr/bin/env bash
# Detect available orchestration frameworks
check_dir_or_file() { [ -e "$1" ] && echo "true" || echo "false"; }
KBD=$(check_dir_or_file ".kbd-orchestrator/project.json")
EVOLVER=$(check_dir_or_file ".evolver/")
REFINER=$(check_dir_or_file ".refiner/")
OPENSPEC=$(check_dir_or_file "openspec/")
echo "{\"kbd\":{\"available\":$KBD},\"evolver\":{\"available\":$EVOLVER},\"refiner\":{\"available\":$REFINER},\"openspec\":{\"available\":$OPENSPEC}}"
