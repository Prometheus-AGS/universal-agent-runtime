# Baseline verdict confirmation (round 3)

The boundary runner must prove it can tell old behavior from new: every mandatory scenario
is run against the pre-phase baseline (revision `f2c9e36efe088e06a17c5d880ada877e25f2ad52`)
and its verdict there must match a frozen expectation. A wrong expectation rejects a correct
runner, so each owner confirms their own scenarios from source.

For every **mandatory** scenario in your changes, write
`openspec/changes/<change-id>/fixtures/baseline-verdicts.json`:

```json
{
  "baselineRevision": "f2c9e36efe088e06a17c5d880ada877e25f2ad52",
  "scenarios": {
    "SCN-XXX-001": {
      "baseline": "FAIL",
      "kind": "new-behavior",
      "evidence": "src/path.rs:120-134 — today X happens, the oracle requires Y"
    },
    "SCN-XXX-002": {
      "baseline": "PASS",
      "kind": "regression-guard",
      "evidence": "…"
    },
    "SCN-XXX-003": {
      "baseline": "NOT_RUNNABLE",
      "kind": "new-behavior",
      "evidence": "fixture needs an endpoint the baseline lacks (…)"
    }
  }
}
```

- `baseline`: `FAIL` (new behavior the baseline lacks), `PASS` (property already holds:
  regression guard), or `NOT_RUNNABLE` (the baseline cannot even set up the fixture; the
  runner records the setup failure and that counts as "baseline lacks it").
- `evidence`: the file:line you read at the baseline revision (`git show f2c9e36e:<path>`
  is allowed for reading). No builds, no tests.
- If a scenario's verdict is PASS, say in the row what proves the runner is not trivially
  passing it (its mutant or rejecting negative control).

Allowed commands: reading files, `git show f2c9e36e:<path>`, grep, the OpenSpec runner
(`validate --strict`). Reply in under 150 words with counts of FAIL / PASS / NOT_RUNNABLE
and any scenario you could not decide.
