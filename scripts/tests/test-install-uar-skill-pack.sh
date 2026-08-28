#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/uar-skill-pack-test.XXXXXX")"
cleanup() {
    rm -rf "$work"
}
trap cleanup EXIT HUP INT TERM

fake_cargo="$work/fake-cargo"
cargo_log="$work/cargo.log"
cat >"$fake_cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
: "${CARGO_TARGET_DIR:?CARGO_TARGET_DIR is required}"
: "${UAR_SKILL_PACK_CARGO_LOG:?UAR_SKILL_PACK_CARGO_LOG is required}"
printf '%s\n' "$*" >"$UAR_SKILL_PACK_CARGO_LOG"
mkdir -p "$CARGO_TARGET_DIR/release"
cat >"$CARGO_TARGET_DIR/release/prometheus" <<'BIN'
#!/usr/bin/env bash
printf 'prometheus test binary\n'
BIN
chmod +x "$CARGO_TARGET_DIR/release/prometheus"
EOF
chmod +x "$fake_cargo"

prefix="$work/home/.config/uar/skills"
source_checkout="$root/crates/prometheus-skill-system"
pinned_source="$work/pinned-skill-pack-source"
"$root/scripts/tests/materialize-pinned-skill-pack-source.sh" \
    "$source_checkout" "$pinned_source" >/dev/null
CARGO="$fake_cargo" \
UAR_SKILL_PACK_CARGO_LOG="$cargo_log" \
    "$root/scripts/install-uar-skill-pack.sh" \
    --source-dir "$pinned_source" \
    --prefix "$prefix"

version="$(awk -F'"' '/"version"[[:space:]]*:/ { print $4; exit }' \
    "$pinned_source/.claude-plugin/plugin.json")"
installed="$prefix/prometheus-skill-pack/$version"

[[ -d "$installed/skills" ]]
[[ -f "$installed/.claude-plugin/plugin.json" ]]
[[ -x "$installed/bin/prometheus" ]]
[[ "$(cat "$installed/UAR_PACK_COMMIT")" == "c25561548aeb9ca656fdb942ab34378beedc2fe2" ]]
grep -q -- '--locked --release' "$cargo_log"
grep -q -- '-p prometheus-cli' "$cargo_log"

source_count="$(find "$pinned_source/skills" -name SKILL.md -type f | wc -l | tr -d ' ')"
installed_count="$(find "$installed/skills" -name SKILL.md -type f | wc -l | tr -d ' ')"
[[ "$installed_count" == "$source_count" ]]

printf 'clean-prefix install PASS: version=%s skills=%s\n' "$version" "$installed_count"

wrong_source="$work/wrong-source"
git clone --quiet --no-local "$pinned_source" "$wrong_source"
git -C "$wrong_source" config user.email test@example.invalid
git -C "$wrong_source" config user.name "Skill pack installer test"
printf 'wrong commit\n' >"$wrong_source/INSTALLER_NEGATIVE_CONTROL"
git -C "$wrong_source" add INSTALLER_NEGATIVE_CONTROL
git -C "$wrong_source" commit --quiet -m "test: wrong installer commit"
if CARGO="$fake_cargo" UAR_SKILL_PACK_CARGO_LOG="$cargo_log" \
    "$root/scripts/install-uar-skill-pack.sh" \
    --source-dir "$wrong_source" \
    --prefix "$work/wrong-prefix" >"$work/wrong.out" 2>&1; then
    printf 'wrong-commit control unexpectedly passed\n' >&2
    exit 1
fi
grep -q 'does not match UAR pin' "$work/wrong.out"
printf 'wrong-commit negative control PASS\n'

failing_cargo="$work/failing-cargo"
cat >"$failing_cargo" <<'EOF'
#!/usr/bin/env bash
exit 42
EOF
chmod +x "$failing_cargo"
if CARGO="$failing_cargo" \
    "$root/scripts/install-uar-skill-pack.sh" \
    --source-dir "$pinned_source" \
    --prefix "$work/failed-build-prefix" >"$work/build.out" 2>&1; then
    printf 'failed-build control unexpectedly passed\n' >&2
    exit 1
fi
[[ ! -e "$work/failed-build-prefix/prometheus-skill-pack/$version" ]]
printf 'failed-build negative control PASS\n'
