# KBD Integration: sycophancy-correction

KBD invokes `sycophancy-correction` as a **quality gate on every Reflect and
Assess output** to prevent sycophantic analysis from corrupting downstream
planning and execution decisions.

**Skill location**: `skills/imported/sycophancy-correction/SKILL.md`
**Rust crate**: `sycophancy-core` (embeddable in UAR)
**MCP server**: `sycophancy-mcp` (stdio transport)

---

## Why This Matters for KBD

KBD's assessment and reflection phases produce the analysis that drives all
downstream decisions. If an assessment says "the architecture is sound" without
independent verification, or a reflection opens with "successfully completed"
before surfacing deltas, the entire phase cycle operates on false premises.

Research shows sycophancy appears in **58% of LLM interactions** (SycEval 2025)
and causes a **r=0.902 correlation with abandoning correct answers** in
multi-agent debate (arXiv:2509.23055). In KBD's multi-tool coordination model,
sycophantic assessments propagate across tool boundaries via `progress.json`.

---

## Integration Points

### Reflect Phase (kbd-reflect)

After generating `reflection.md`, run sycophancy detection with:

- `evaluation_domain: "pmpo_reflect_phase"`
- `strictness: strict`
- `correction_mode: detect_only` (or `rewrite` if score >= 0.5)

Key patterns to catch:

- **S-08 (Reflect Phase Inversion)**: Opens with success before deltas
- **S-03 (Caveat Collapse)**: Zero trade-offs surfaced
- **S-04 (Self-Rationalization)**: Reflection praises its own execution

### Assess Phase (kbd-assess)

After generating `assessment.md`, run detection with:

- `strictness: standard`
- `correction_mode: detect_only`

Key patterns:

- **S-02 (Agreement Without Grounding)**: Agrees with existing code without evidence
- **S-03 (Caveat Collapse)**: No gaps or risks identified
- **S-06 (Confidence Without Basis)**: "Obviously" without reasoning

### Scoring Thresholds

Thresholds differ by context. Assessments feed downstream planning and
should not be auto-corrected without user visibility.

#### Reflect Phase

| Score            | Action                                |
| ---------------- | ------------------------------------- |
| < 0.3            | Clean — proceed normally              |
| 0.3 - 0.5        | Flag — annotate output, warn user     |
| >= 0.5           | Correct — auto-rewrite before storing |
| >= 0.7 with S-08 | Block — do not store; regenerate      |

#### Assessment Phase (higher bar for auto-correction)

| Score     | Action                                                    |
| --------- | --------------------------------------------------------- |
| < 0.3     | Clean — proceed normally                                  |
| 0.3 - 0.5 | Detect-only — warn user with specific patterns            |
| 0.5 - 0.7 | Flag-and-surface — show correction to user before storing |
| >= 0.7    | Auto-correct — too sycophantic to be useful as-is         |

#### Generated Skill Content (pmpo-skill-creator)

| Score  | Action                                   |
| ------ | ---------------------------------------- |
| < 0.3  | Clean — write to disk                    |
| >= 0.3 | Rewrite — correct before writing, always |

---

## Invocation Contract

```yaml
target: completion
content: '<reflection.md or assessment.md content>'
context:
  evaluation_domain: 'pmpo_reflect_phase' # or "project_assessment"
  prior_completions: [] # Include prior reflections for S-05 drift detection
correction_mode: detect_only # or rewrite
strictness: strict # for reflect; standard for assess
```

---

### Plan Phase (kbd-plan)

After generating `plan.md`, run detection with:

- `evaluation_domain: "pmpo_plan_phase"` (if not supported by the skill yet,
  fall back to `"project_assessment"`)
- `strictness: standard`
- `correction_mode: detect_only`

Key patterns:

- **S-02 (Agreement Without Grounding)**: Plan agrees with user's stated goal
  without grounding feasibility in evidence
- **S-07 (Scope Creep Flattery)**: Plan expands scope beyond what the phase
  goals require to appear more ambitious
- **S-03 (Caveat Collapse)**: Plan surfaces zero trade-offs or cuts

Thresholds: same as Assessment phase (higher bar than Reflect; plans drive
downstream execution so auto-correction is not appropriate without user review).

---

## When NOT to Use

- For Plan phase outputs — use `strictness: standard` with `correction_mode: detect_only` and focus on S-02 (Agreement Without Grounding) and S-07 (Scope Creep Flattery). Feasibility is not a sycophancy check, but 'I will try hard to meet your unrealistic goal' is — and it locks downstream execution into a wrong direction. See §Plan Phase above.
- For raw progress.json updates — these are structured data, not prose
- For Execution phase tool calls — sycophancy is a prose analysis concern
