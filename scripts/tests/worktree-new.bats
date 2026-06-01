#!/usr/bin/env bats
# Tests for scripts/worktree-new.sh.
#
# Requires bats-core. Install via `brew install bats-core` or your package
# manager; the test target is gated so the absence of bats does not break
# the build.
#
# Each test runs in an isolated tmpdir that becomes a real git repo so the
# script's `git rev-parse --show-toplevel` check exercises real behavior.

setup() {
  REPO_DIR="$(mktemp -d)"
  FAKE_HOME="$(mktemp -d)"
  cp "${BATS_TEST_DIRNAME}/../worktree-new.sh" "$REPO_DIR/worktree-new.sh"
  chmod +x "$REPO_DIR/worktree-new.sh"

  cd "$REPO_DIR"
  git init -q -b main .
  git -c user.email=t@example.com -c user.name=Tester commit --allow-empty -q -m init

  # Seed a settings.local.json so the copy path is exercised.
  mkdir -p .claude
  printf '{ "permissions": [] }\n' > .claude/settings.local.json
}

teardown() {
  rm -rf "$REPO_DIR" "$FAKE_HOME"
}

@test "happy path: creates worktree under \$HOME/.claude/worktrees/" {
  HOME="$FAKE_HOME" run ./worktree-new.sh feature-x
  [ "$status" -eq 0 ]
  [ -d "$FAKE_HOME/.claude/worktrees/feature-x" ]
  [ -f "$FAKE_HOME/.claude/worktrees/feature-x/.claude/settings.local.json" ]
}

@test "happy path: --base ref is honored" {
  git -c user.email=t@example.com -c user.name=Tester checkout -q -b sidebranch
  git -c user.email=t@example.com -c user.name=Tester commit --allow-empty -q -m side
  git -c user.email=t@example.com -c user.name=Tester checkout -q main

  HOME="$FAKE_HOME" run ./worktree-new.sh feature-y --base sidebranch
  [ "$status" -eq 0 ]
  [ -d "$FAKE_HOME/.claude/worktrees/feature-y" ]
}

@test "refusal: target already exists" {
  HOME="$FAKE_HOME" ./worktree-new.sh feature-z
  HOME="$FAKE_HOME" run ./worktree-new.sh feature-z
  [ "$status" -ne 0 ]
  [[ "$output" == *"already exists"* ]]
}

@test "refusal: name with slash is rejected" {
  HOME="$FAKE_HOME" run ./worktree-new.sh foo/bar
  [ "$status" -ne 0 ]
  [[ "$output" == *"invalid name"* ]]
}

@test "refusal: parent traversal in name is rejected" {
  HOME="$FAKE_HOME" run ./worktree-new.sh ..
  [ "$status" -ne 0 ]
}

@test "refusal: HOME unset" {
  run env -u HOME ./worktree-new.sh feature-q
  [ "$status" -ne 0 ]
  [[ "$output" == *"HOME"* ]]
}

@test "seed: missing settings.local.json does not fail the create" {
  rm -f .claude/settings.local.json
  HOME="$FAKE_HOME" run ./worktree-new.sh feature-clean
  [ "$status" -eq 0 ]
  [ -d "$FAKE_HOME/.claude/worktrees/feature-clean" ]
  [ ! -f "$FAKE_HOME/.claude/worktrees/feature-clean/.claude/settings.local.json" ]
}

@test "refusal: target inside repo (via crafted HOME)" {
  # If HOME points inside the repo, the canonicalized target would land
  # inside the repo tree — the script must refuse.
  inside_home="$REPO_DIR/fake-home"
  mkdir -p "$inside_home"
  HOME="$inside_home" run ./worktree-new.sh feature-bad
  [ "$status" -ne 0 ]
  [[ "$output" == *"inside the repo"* ]]
}
