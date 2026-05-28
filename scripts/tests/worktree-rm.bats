#!/usr/bin/env bats
# Tests for scripts/worktree-rm.sh.

setup() {
  REPO_DIR="$(mktemp -d)"
  FAKE_HOME="$(mktemp -d)"
  cp "${BATS_TEST_DIRNAME}/../worktree-new.sh" "$REPO_DIR/worktree-new.sh"
  cp "${BATS_TEST_DIRNAME}/../worktree-rm.sh"  "$REPO_DIR/worktree-rm.sh"
  chmod +x "$REPO_DIR/worktree-new.sh" "$REPO_DIR/worktree-rm.sh"

  cd "$REPO_DIR"
  git init -q -b main .
  git -c user.email=t@example.com -c user.name=Tester commit --allow-empty -q -m init

  HOME="$FAKE_HOME" ./worktree-new.sh w-target
}

teardown() {
  rm -rf "$REPO_DIR" "$FAKE_HOME"
}

@test "happy path: removes a worktree under HOME/.claude/worktrees/" {
  HOME="$FAKE_HOME" run ./worktree-rm.sh w-target --force
  [ "$status" -eq 0 ]
  [ ! -d "$FAKE_HOME/.claude/worktrees/w-target" ]
}

@test "refusal: missing worktree" {
  HOME="$FAKE_HOME" run ./worktree-rm.sh nope
  [ "$status" -ne 0 ]
  [[ "$output" == *"no such worktree"* ]]
}

@test "refusal: name with slash is rejected" {
  HOME="$FAKE_HOME" run ./worktree-rm.sh foo/bar
  [ "$status" -ne 0 ]
  [[ "$output" == *"invalid name"* ]]
}
