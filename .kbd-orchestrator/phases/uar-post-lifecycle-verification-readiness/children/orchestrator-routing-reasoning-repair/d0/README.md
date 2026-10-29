# D0 — classifier reply shape (operator-approved, 2026-10-27)

The script was shown to the operator before running: `d0-classifier-shape.py` in the session scratchpad. It is not copied into the repository because it reads the gateway key from the service secrets file. Command: `python3 -I <scratchpad>/d0-classifier-shape.py`. Output is shape-only, with no reply text. Each of the 3 calls printed: `separate_reasoning_field: false, finish_reason: "stop", completion_tokens: 10, raw_chars: 31, think_open: 0, think_close: 0, raw_trim_is_eligible_id: true, answer_is_expected: true, normalized_is_expected: true, answer_wrapper_chars: []` (7 candidates).

## Result

Three non-streaming MiniMax-M3 calls through Liter, using the router's system/user prompt, the C34 roster plus the houseplant candidate, and `max_completion_tokens` 1000. Each returned **exactly** `disposable-plant-clinic-d0probe`: 10 completion tokens, finish `stop`, no `<think>`, no wrapper characters, no separate `reasoning_content`. **The failure did not reproduce in this approximation**, which is non-streaming, has placeholder candidate revisions and lacks UAR prompt wrappers.

## Product-path evidence (read-only: UAR log and run inspection)

- Both failed orchestrator runs had **no active skills** (`GET /api/uar/runs/b3b12923…` and `…/5297800c…`: `activated_skills: []`). `SkillRequestDriver` re-attachment (`src/uar/runtime/skills/usage.rs:97-115`) therefore added nothing.
- Classifier stream sizes in the UAR log, each the stream completing just before the selection or error:

| Route | Log line | Stream | Outcome |
|---|---|---|---|
| attempt 1, eligible specialist | 1436 | 5 chunks / 3 events | selected |
| attempt 2, eligible specialist | 1478 | 6 / 4 | selected |
| attempt 2, fresh task after removal | 1513 | 27 / 26 | `graph_routing_selection_invalid` |
| attempt 2, unmatched intent | 1532 | 5 / 3 | selected (default-agent) |
| attempt 3, eligible specialist | 1568 | 19 / 17 | `graph_routing_selection_invalid` |

## Interpretation

Failures coincide with classifier replies 4–8× longer than an ID-only reply in the **streaming** product path. Non-streaming D0 calls never produced the extra text. Its nature (inline reasoning versus prose such as "The best match is …") is **still unobserved**. T2's content-free diagnostics (`answer_chars`, `had_reasoning`, `unclosed`, `normalized_changed`) are what will identify it. If it is prose, T2 still refuses (no fuzzy matching) and C34 fails again with that diagnostic, which then decides the next step. The operator's instruction to implement T1–T7 after D0 stands; D0 does not contradict the plan, it narrows the uncertainty.
