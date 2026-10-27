# A15 / independent diagnostic sweep / one correction per item

PROPOSED, UNSIGNED, NOT ADOPTED, NOT EXECUTED. A14 binding16 remains adopted. Full product verification CLOSED. This is the single exact exception for the independent diagnostic sweep before deferred fixture completion.

## Operator rule

For each named item: PASS → record it and continue. FAIL → diagnose the observed failure, make at most one relevant correction, refresh source inventory, and rerun that same item once. Still FAIL → append failure/backlog row and continue independent items. No repair/retry loop and no new permission request per item within this exact signed scope. An unavailable prerequisite is BLOCKED, never PASS or a fabricated product failure. Unknown/exhausted global allowance or an unresolved process/credential/authority boundary still stops execution.

The uncomfortable thing: the fixture correction actually failed again, so most user-visible acceptance remains unexercised. A15 collects additional independent failures; it cannot establish product suitability, storage parity or release validation. Passing checks on an earlier source snapshot are not silently carried forward over later corrections.

## Current results and exact scope

| Group | Count | Disposition |
| --- | --- | --- |
| Fixture preparation |1|FAIL_AFTER_ONE_CORRECTION; no additional fixture correction under A15 |
| User outcomes |97cases/582backend repetitions|BLOCKED: composed fixture absent |
| Main supporting acceptance |10scenarios|BLOCKED: supporting manifest/receipt incomplete |
| Issue426 integration |4targets|BLOCKED: isolated supporting-store input set incomplete; no shared-store fallback or silently skipped remote cases |
| Supported profiles |6|Queued diagnostic items |
| OpenSpec validation |8|Queued diagnostic items |
| Independent source reviews |2|Queued diagnostic items; no runtime-review claim |
| Review screens |2|Queued; depend only on their actual report |

A14 signature/adoption succeeded. M0.002 admitted the retained parent and charged repair10. M1.002 failed while binding nonregular source metadata with null digest. Supporting env/config/store/docs exist; manifest/receipt/resource002/seed002/composition do not. Exactly3replacement supporting JWTs were persisted privately; original2product JWTs and keys retained. No services/provider/model/embedding calls or product tests ran. Further fixture work is backlogged, not retried. [Append-only result log](/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-verification-results.jsonl) preserves this result and the original mkdir/order failures.

Source remains `sha256:28137dc46cbb05aa9169f93b46232ea6aafc8d3f04e7c841274e4bbf4d109018` at the start. This turn changed governance/target metadata only, not production source. Canonical2.9 remains incomplete. The precise unfinished work is supporting manifest/receipt, resource002/derived inventory/composition and final activation. Source inspection cannot establish actual compiler/spec-checker outcomes; A15 explicitly authorizes only named independent checks before that boundary. No test runner is invoked merely to rediscover the known missing fixture.

## Contract and finite repair budget

A15 contract `02942ff1e5adafd5316183b09e8fea103238388efea8ccfba346e8d4760d2cc3` supersedes A14 only after actual signature/adoption. [Unsigned amendment](/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.independent-sweep.amendment.json) generation exited0 with0scenario additions/modifications/removals; all111 obligations remain.

Propose cumulative repair cap **10→28**:10 already consumed, plus at most18 one-per-item diagnostic slots. Unused slots do not permit a second correction to another item. Fixture has zero additional slots. Charge each slot on correction start, not on successful completion. No dependency upgrades/system installs, broad refactors, test/spec weakening or bypassing the policy validator. Only minimum source/config/lock correction directly explaining the named observed failure is allowed. If no legitimate correction exists within scope, record the reason and continue.

Keep all prior epoch, consumption, overruns, UNKNOWNs,4,000 cumulative calls,2,400 conservative active minutes,no-progress3, output/file/tree/token ceilings and separate USD10 Cohere/USD10 OpenAI caps. All later work and waits accrue under existing exclusions only. No provider calls/spend in this independent sweep, no secrets copied or inherited. Proposed batch maximum28,800seconds (8hours), clamped to actual remaining global allowance; setup/signing/fix time included. This is not a new global allowance. Builds have timeout1,800seconds, other commands120seconds; each may run at most twice, second run only after its one targeted correction. Source reviews have the same once-plus-one-correction rule, no command execution by reviewers.

## Exact future signature/adoption

Only after A15 approval, repository cwd `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`:

```bash
ssh-keygen -Y sign -f /Users/gqadonis/.ssh/id_ed25519.pub -n prometheus-kbd-scope-amendment .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.independent-sweep.amendment.json
bash /Users/gqadonis/Projects/prometheus/prometheus-skill-system/shared/scripts/kbd-contract-adopt.sh .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.independent-sweep.proposed.json --supersedes contract-adoption-uar-post-lifecycle-verification-readiness-cb7b76fe --amendment .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.independent-sweep.amendment.json --signature .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.independent-sweep.amendment.json.sig
```

Use registered SSH-agent signer/namespace; no private-key read or trust change. A15 explicitly supersedes older first-failure and per-edit new-approval wording only for these18named items. Every correction still gets a refreshed inventory and recorded same-item instruction before rerun. This combined approval supplies that bounded instruction in advance, eliminating a per-defect signature cycle. No fixture/service/product gate is opened by it.

## Exact serial commands and reviews

Order: S5–S10 profiles → S14–S21 specs → RUST → CODE → S12/S13 screens. All execution local. Exclusive output root `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-diagnostic-sweep-001` and evidence root `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001`; retain old targets/evidence. Child environment includes only named existing P0 toolchain fields, HOME=/Users/gqadonis, CARGO_TARGET_DIR=/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-diagnostic-sweep-001/cargo and DOCKER_HOST=unix:///Users/gqadonis/.colima/default/docker.sock. No ambient Docker context or provider keys. Capture bounded stdout/stderr, actual exit and artifact identity, then move on. Do not execute extra readiness/version/import/probe/check commands.

### S5

```bash
cargo build --locked --no-default-features --features minimal --bin universal-agent-runtime
```

Timeout≤1800seconds; maximum2attempts with at most1targeted correction.

Separate serial gate; retain artifact identity before later overwrite. Local build only, no push/run/deploy.

### S6

```bash
cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime
```

Timeout≤1800seconds; maximum2attempts with at most1targeted correction.

Separate serial gate; retain artifact identity before later overwrite. Local build only, no push/run/deploy.

### S7

```bash
cargo build --locked --no-default-features --features embedded-mobile --lib
```

Timeout≤1800seconds; maximum2attempts with at most1targeted correction.

Separate serial gate; retain artifact identity before later overwrite. Local build only, no push/run/deploy.

### S8

```bash
cargo build --locked --no-default-features --features embedded-mobile --lib --target aarch64-apple-ios
```

Timeout≤1800seconds; maximum2attempts with at most1targeted correction.

Separate serial gate; retain artifact identity before later overwrite. Local build only, no push/run/deploy.

### S9

```bash
cargo build --locked --no-default-features --features embedded-mobile --lib --target aarch64-linux-android
```

Timeout≤1800seconds; maximum2attempts with at most1targeted correction.

Separate serial gate; retain artifact identity before later overwrite. Local build only, no push/run/deploy.

### S10

```bash
docker build --platform linux/amd64 --tag uar-issue-426:1.3.0 .
```

Timeout≤1800seconds; maximum2attempts with at most1targeted correction.

Separate serial gate; retain artifact identity before later overwrite. Local build only, no push/run/deploy.

local linux/amd64 image build tag uar-issue-426:1.3.0; Dockerfile/build scripts and dependency/container registry network, no push/run/publish/deploy; no images deleted

### S14

```bash
openspec validate pl-main-budget-reconciliation
```

Timeout≤120seconds; maximum2attempts with at most1targeted correction.

### S15

```bash
openspec validate pl-main-compatible-budget-errors
```

Timeout≤120seconds; maximum2attempts with at most1targeted correction.

### S16

```bash
openspec validate pl-main-destination-budget-contract
```

Timeout≤120seconds; maximum2attempts with at most1targeted correction.

### S17

```bash
openspec validate pl-main-persistence-discriminators
```

Timeout≤120seconds; maximum2attempts with at most1targeted correction.

### S18

```bash
openspec validate pl-main-boundary-evidence
```

Timeout≤120seconds; maximum2attempts with at most1targeted correction.

### S19

```bash
openspec validate openai-client-tools
```

Timeout≤120seconds; maximum2attempts with at most1targeted correction.

### S20

```bash
openspec validate update-parking-lot-admission
```

Timeout≤120seconds; maximum2attempts with at most1targeted correction.

### S21

```bash
openspec validate reengineer-user-outcome-verification
```

Timeout≤120seconds; maximum2attempts with at most1targeted correction.

### RUST

Fresh isolated `rust-reviewer` receives current source/diff/contracts and actual available build/spec receipts, without generation history. It writes `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-diagnostic-sweep-001/reviews/whole-phase-rust-review.md`. It may inspect artifacts/source but run no checks. Disclose absent runtime evidence. If it fails, one targeted source correction and one fresh isolated review of the corrected artifacts; retain both reports.

### CODE

Fresh isolated `code-reviewer` receives current source/diff/contracts and actual available build/spec receipts, without generation history. It writes `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-diagnostic-sweep-001/reviews/whole-phase-code-review.md`. It may inspect artifacts/source but run no checks. Disclose absent runtime evidence. If it fails, one targeted source correction and one fresh isolated review of the corrected artifacts; retain both reports.

### S12

```bash
prometheus sycophancy detect /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-diagnostic-sweep-001/reviews/whole-phase-rust-review.md --strictness standard
```

Timeout≤120seconds; maximum2attempts with at most1targeted correction.

### S13

```bash
prometheus sycophancy detect /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-diagnostic-sweep-001/reviews/whole-phase-code-review.md --strictness standard
```

Timeout≤120seconds; maximum2attempts with at most1targeted correction.

## Source refresh, evidence and continuation

Named metadata command after a correction, with ITEM substituted only by its stable listed ID:

```bash
/Users/gqadonis/miniconda3/bin/python3 /Users/gqadonis/Projects/prometheus/universal-agent-runtime/scripts/acceptance/source_inventory.py --output /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/ITEM/source-after-one-correction.json
```

Record before/after source identities, exact patch/root cause/repair ordinal and exact item rerun. One writer per file/target, no overlap. Relevant specialized artifact-only code review may inspect the correction; no hidden extra gate. Keep original source/build claims separate from new source. No rerun of a previously passing different item is authorized; report those results with their original identities and the resulting limitations.

The [frozen packet](/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-independent-sweep-001.packet.json) enumerates18items, all metadata/evidence effects and exclusions. Its file digest is `sha256:4d84966d4bed961da08d6b784cfcb61eac8b24a7e445d08c8f15eee99d771fb0`. Ledger rows contain item, attempt, source, command, exit, outcome, artifacts, correction used, remaining budget and next dependency disposition. Final table separates PASS, PASS_AFTER_ONE_CORRECTION, FAIL, FAIL_AFTER_ONE_CORRECTION, BLOCKED and NOT_RUN. Archive/reflection/release/full-validation claims remain prohibited while exit criteria fail.

## Exact approval

> Approve A15 signing/adoption and the18-item independent diagnostic sweep exactly in this packet. Raise repair cap10to28 solely for one targeted correction and one same-item rerun per named item; fixture remains failed after its single correction with no further repair. Preserve all prior consumption/other limits and USD10 per-provider caps. Run serially, log unresolved failures and continue independent items. Authorize fresh source inventory and same-item rerun after each bounded correction without a new per-defect approval/signature. Stop only for unknown/exhausted global allowance or unresolved process/credential/authority boundary. No new fixture/service/provider/product test/publication operation, system install, dependency upgrade, policy bypass or acceptance weakening.

One combined fresh signature/instruction is required because A14's signed first-failure/repair cap and AGENTS.md's closed whole-phase gate do not authorize this new independent sweep. This packet explicitly names the diagnostic exception, unfinished implementation and limits; approval replaces repeated per-failure permission requests within its finite scope.
