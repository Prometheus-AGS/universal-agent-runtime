#!/usr/bin/env bash
# Operator instruction 2026-11-22 ("Finish the run"): run boundary commands past a FAIL, one runner call per index,
# to find every failure. It changes no command, judges nothing and reruns nothing already recorded for this commit.
# Usage, from the repository root: run-past-failures.sh <first> <last>
# Exit: 0 all ran, 7 binding broken, 8 disk low, 9 refused (index already recorded).
set -uo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
E=$P/boundary/evidence/attempt-4-continue
C=$(git rev-parse --short=8 HEAD)
[ "$C" = 9ad39030 ] || { echo "HEAD is $C, not 9ad39030"; exit 7; }
mkdir -p "$E"

sample() {
  local free idle others
  free=$(df -g /Volumes/T7 | tail -1 | awk '{print $4}')
  idle=$(top -l 2 -n 0 -s 1 | grep "CPU usage" | tail -1 | awk '{print $7}')
  others=$(for p in $(pgrep -x cargo); do lsof -a -p "$p" -d cwd -Fn 2>/dev/null | sed -n 's/^n//p'; done | tr '\n' ',')
  echo "$(date +%FT%T) index=$1 idle=$idle swap=$(sysctl -n vm.swapusage | awk '{print $6"/"$3}') t7free=${free}G others=${others:-none}" >> "$P/boundary/attempt-4-load.log"
}

save() {
  local out=$1 stamp=$2 src name
  for src in tests/bdd/test-results tests/bdd/cucumber-report.json frontend/test-results frontend/playwright-report \
             "$HOME/.claude/worktrees/s1-mutant-m1/tests/bdd/test-results" \
             "$HOME/.claude/worktrees/s1-mutant-m2/tests/bdd/test-results"; do
    [ -e "$src" ] || continue
    [ -n "$(find "$src" -newer "$stamp" -print -quit 2>/dev/null)" ] || continue
    name=$(printf '%s' "${src#"$HOME"/.claude/worktrees/}" | tr '/' '_')
    mkdir -p "$out"
    cp -c -R "$src" "$out/$name"
  done
}

for i in $(seq "$1" "$2"); do
  if jq -e -s --argjson i "$i" --arg c "$C" 'any(.[]; .commit==$c and .index==$i)' "$P/boundary/run.jsonl" >/dev/null; then
    echo "index $i is already recorded for $C; not rerunning"; exit 9
  fi
  if [ "$(git rev-parse --short=8 HEAD)" != "$C" ] || [ -n "$(git status --porcelain --untracked-files=all -- . ':(exclude,glob)**/.kbd-orchestrator/**' ':(exclude,glob)**/.prometheus/**' ':(exclude)static')" ]; then
    echo "binding broken before index $i"; exit 7
  fi
  [ "$(df -g /Volumes/T7 | tail -1 | awk '{print $4}')" -gt 4 ] || { echo "disk low before index $i"; exit 8; }
  stamp="$E/.stamp-$i"; : > "$stamp"
  sample "$i"
  env -u UAR_STATIC_DIR -u UAR_GIT_SHA uv run --no-project python "$P/boundary/tools/boundary-runner.py" "$i-$i" &
  runner=$!
  while kill -0 "$runner" 2>/dev/null; do
    for _ in 1 2 3 4 5; do kill -0 "$runner" 2>/dev/null || break; sleep 4; done
    sample "$i"
  done
  if ! wait "$runner"; then
    save "$E/$i" "$stamp"
    echo "index $i failed; evidence in $E/$i"
  fi
done
echo "ran $1 to $2"
