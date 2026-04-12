#!/usr/bin/env bash
check() { [ -e "$1" ] && echo "true" || echo "false"; }
KBD=$(check ".kbd-orchestrator/project.json")
KBD2=$(check ".claude/skills/kbd-process-orchestrator/")
EVOLVER=$(check ".evolver/")
REFINER=$(check ".refiner/")
OPENSPEC=$(check "openspec/")
[ "$KBD" = "true" ] || KBD=$KBD2
echo "{\"kbd\":{\"available\":$KBD},\"evolver\":{\"available\":$EVOLVER},\"refiner\":{\"available\":$REFINER},\"openspec\":{\"available\":$OPENSPEC}}"
