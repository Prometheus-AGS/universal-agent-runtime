
## Worktree provisioning

- Repo: `prometheus-entity-management`
- Worktree: `~/.claude/worktrees/seim-entity-management`
- Branch: `feat/seim-entity-management-impl`
- Base commit: `1abae4f74b3c3e2b22a0c2f7ef18e931a89a81fd` (from `origin/main` at provisioning time)
- Provisioned by: `seim-em-worktree-setup`

Subsequent changes that write code to entity-management MUST resolve
the worktree path from `worktrees.json` (sibling to `progress.json`)
and refuse to operate if the entry is absent.
