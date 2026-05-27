# Implementation Tasks — ssed-kbd-agent-rules-injector

> Target: `prometheus-skill-system`. Cache file pre-populated from web search performed during proposal/design.

## 1. Skill files

- [x] 1.1 Directory + SKILL.md + kbd-inject-agent-rules.sh + references/rules-cache.md + references/template.md
- [x] 1.2 `chmod +x`; syntax check

## 2. Cache + template

- [x] 2.1 `references/rules-cache.md` — both rule sets verbatim, ISO fetch dates, source URLs + anchor keywords
- [x] 2.2 `references/template.md` — fenced region body

## 3. Implementation behaviors

- [x] 3.1 Argument parsing (--target, --path, --refresh, --dry-run)
- [x] 3.2 Marker detection (start/end pairs; refuse on missing-end / multi-start)
- [x] 3.3 awk-based fenced-region replacement
- [x] 3.4 First-write append (no markers present)
- [x] 3.5 Idempotency — second run leaves bit-identical file
- [x] 3.6 --dry-run prints `diff -u`
- [x] 3.7 --refresh curl-probes anchors; warns on failure but proceeds

## 4. Smoke tests

- [x] 4.1 `test-agent-rules-injector.sh` with cases for first-write, replace-in-place, idempotency, --dry-run, missing-end refusal, multi-start refusal, --target single, --target both
- [x] 4.2 All cases pass live

## 5. First-customer

- [x] 5.1 Run against this UAR worktree's CLAUDE.md + AGENTS.md (with rollback prep)
- [x] 5.2 Confirm fenced region present in both, identical body
- [x] 5.3 Confirm second run = bit-identical (idempotency)

## 6. Closeout

- [ ] 6.1 `/opsx:verify` + `/opsx:archive`
- [ ] 6.2 progress.json `changes_completed: 7`; active_change → `ssed-uar-uiux-skill-routing`
