#!/usr/bin/env bash
set -euo pipefail

fail() {
    printf 'pinned skill-pack fixture: %s\n' "$*" >&2
    exit 1
}

if [[ $# -ne 2 ]]; then
    fail "usage: $0 <initialized-source-checkout> <destination>"
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
installer="$root/scripts/install-uar-skill-pack.sh"
source_dir="$(cd "$1" && pwd)"
destination="$2"
pin="$(awk -F'"' '$1 == "readonly PACK_COMMIT=" { print $2; exit }' "$installer")"

[[ "$pin" =~ ^[0-9a-f]{40}$ ]] || fail "could not read the installer commit pin"
git -C "$source_dir" rev-parse --is-inside-work-tree >/dev/null 2>&1 \
    || fail "$source_dir is not a Git checkout"
git -C "$source_dir" cat-file -e "$pin^{commit}" 2>/dev/null \
    || fail "source checkout does not contain installer commit $pin"
[[ ! -e "$destination" ]] || fail "destination already exists: $destination"

if git -C "$source_dir" submodule status --recursive | grep -q '^-'; then
    fail "source checkout has uninitialized submodules; run git submodule update --init --recursive"
fi

git clone --quiet --shared --no-checkout "$source_dir" "$destination"
git -C "$destination" checkout --quiet --detach "$pin"

git_config=(-c protocol.file.allow=always)
seen_urls=$'\n'
while IFS=$'\t' read -r remote local_path; do
    [[ -n "$remote" && -n "$local_path" ]] \
        || fail "could not map an initialized submodule to its local checkout"
    case "$seen_urls" in
        *$'\n'"$remote"$'\n'*) continue ;;
    esac
    seen_urls+="$remote"$'\n'
    git_config+=(-c "url.file://$local_path.insteadOf=$remote")
done < <(
    git -C "$source_dir" submodule foreach --quiet --recursive \
        'printf "%s\t%s\n" "$(git config --get remote.origin.url)" "$(pwd -P)"'
)

(( ${#git_config[@]} > 2 )) \
    || fail "source checkout exposes no initialized submodule mirrors"
git "${git_config[@]}" -C "$destination" \
    submodule update --quiet --init --recursive

actual="$(git -C "$destination" rev-parse HEAD)"
[[ "$actual" == "$pin" ]] || fail "materialized commit $actual does not match $pin"
if git -C "$destination" submodule status --recursive | grep -Eq '^[+-]'; then
    fail "materialized source has missing or mismatched submodules"
fi
[[ -z "$(git -C "$destination" status --porcelain)" ]] \
    || fail "materialized source is not clean"

printf '%s\n' "$destination"
