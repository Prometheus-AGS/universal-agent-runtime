# Task 13.2 policy fixture setup source review

Scope: only newly authored policy copy/binding/receipt changes in `scripts/release/client_tools_local_gate.py` and gate002 handoff addition. No production edits. Current phase remains `uar-post-lifecycle-verification-readiness`, Execute. Verification is CLOSED for this reviewer; lead owns execution.

Reviewed artifact echo:

```python
policy_directory = evidence / "policies"
shutil.copytree(ROOT / "policies", policy_directory)
policies = sorted(policy_directory.glob("*.cedar"))
require(policies, "shipped policy tree contains no Cedar policy files")
report["shipped_policy_files"] = [{"path": str(path.relative_to(evidence)),
    "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in policies]
```

Disposition: no concrete defect found in this bounded source review. Setup is inside the existing failure-reporting try/finally and precedes either child start. Supervisor.start uses `cwd=self.evidence`, so `evidence/policies` matches `GovernanceEngine::load_from_dir("policies")` in server.rs. The engine reads sorted top-level paths with `.cedar` extension; the receipt likewise inventories copied top-level `.cedar` files and hashes copied bytes. Dockerfile copies `/src/policies` into `/opt/uar/policies`, supporting the stated shipped-asset binding. The source bundle contains default.cedar, skill-mutation.cedar, tool-approval.cedar and an excluded `.cedar.example` file. No new permit or governance bypass was introduced by this setup change.

Read the newly changed main-function context, its existing exception/finally reporting, Supervisor.start binding, full policy loading helper, server startup error handling, Dockerfile asset copy and current handoff. Copy/file-read errors reach the existing OSError handler; missing Cedar assets reach GateFailure. Both paths retain failure status and write local-gate.json without requiring a started child.

Only source inspection (`git diff HEAD -- ...`, `rg`, `sed`) and this authorized review-record write ran. Some initial discovery globs named absent paths; subsequent exact reads established the real engine source. No script, import, check, test, startup, build or network execution ran. Successful copying, policy parsing, readiness and the five-interface matrix remain runtime-unverified by this reviewer. Gate002 failure and binary build004 PASS are lead-supplied receipts, not independently reproduced.

Reviewer is an isolated native same-family Codex fallback; no producer generation history was supplied. Different-family certification remains pending. Manual sycophancy screening found no praise quota or unsupported success claim; machine detection was not executed under this source-only assignment and no score is claimed. No guards or implementation outside the requested setup repair were proposed.
