# SEMANTIC-SKILL-01 — Mac PASS

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/semantic-skill.cucumber.mjs
```

Observed: **1 scenario (1 passed), 5 steps (5 passed), 17.395 seconds**. Four Gherkin steps plus cleanup. Only this new scenario ran.

The prompt asked how much to charge for sending boxes to a customer. Neither delivery trigger (`freight tariff`, `consignment valuation`) nor the gardening trigger (`garden watering`) appeared. The fixture also checked the title/description whole-query matches used by the keyword implementation: derived keyword score zero for both. No skill ID, attachment, preference, recipe or private marker appeared in the user request.

Run inspection identified **skill_service.local_embedding** at an explicit **0.65** threshold. The delivery skill scored **0.7355**, was the only accepted/activated skill, and generated an activation event. Both disposable skills were equally eligible through the agent policy. Genuine Liter/MiniMax-M3 inference returned the correct **360** and the random marker stored only in the delivery skill. Cleanup restored the original configuration with readback and deleted both skills and the agent. UAR remains running.

## What failed first

The initial inherited 0.5 threshold admitted both delivery (0.7421) and unrelated gardening (0.5238). The exact-selection assertion failed. The same scenario then passed with an explicitly configured semantic threshold of0.65 and unchanged behavior assertions. This is evidence of a calibrated configuration, not proof that the default threshold is suitable. Both attempts and their raw responses remain intact; no production code or persistent default was changed.

## Files and checks

- `scripts/behaviors/semantic-skill.cucumber.mjs`: selects only the new feature with zero automatic retries.
- `scripts/behaviors/features/semantic-skill.feature`: four behavior steps.
- `scripts/behaviors/steps/semantic-skill.steps.mjs`: disposable setup, temporary matching configuration, zero-keyword checks, genuine inference, selection/activation/private-answer assertions and restoration.
- README and OpenSpec delta document the scenario. Receipt, behavior ledger and append-only memory record actual results.

Source-only general and JavaScript reviews were performed. Review limitations retained: installed skills compete for global embedding top-k before agent policy filtering; runtime admission engine is a separate setting, so service configuration alone is not a portability guarantee; cleanup tracks acknowledged creations, and network-loss recovery after an ambiguous creation is not covered. The actual run confirmed the required engine/strategy and selection, all creates were acknowledged, and all cleanup calls succeeded. These concerns were not converted into unrelated product edits or additional scenarios.

Guards are limited to real HTTP/configuration boundaries, bounded waits, acknowledged cleanup ownership, and a concurrent-config comparison before restoration. No unrelated additions, test authentication, build, restart, prior scenario reruns or publication. Other semantic strategies, arbitrary installed catalogs/paraphrases, default-threshold quality and broad reliability remain unverified. The local matcher uses the existing local embedding resource; no hosted embedding strategy was selected.

[Receipt](semantic-skill-01-macos-pass.json), [raw evidence](semantic-skill/1740f52e-a229-416c-a3f4-9f7243f259e1.json), [passing report](semantic-skill-002-macos/cucumber.json), [failed first report](semantic-skill-001-macos/cucumber.json). Stop for operator direction.
