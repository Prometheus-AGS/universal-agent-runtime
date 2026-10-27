# Goals — uar-post-lifecycle-verification-readiness › orchestrator-routing-reasoning-repair

Opened 2026-10-27 at operator request after C34 attempts 2–3. **Plan only; stop for operator review before implementation.**

- G1 — Registered-agent intent routing selects an eligible agent when the classifier's reply carries a valid ID plus reasoning or formatting. It refuses only a genuinely invalid or ineligible choice, and the refusal is observable enough to tell those cases apart.
- G2 — A delegated (routed) child turn that ends inside reasoning, with no final answer, is not reported as a successful answered run.
- G3 — The user-facing outcomes are proven through the real path by rerunning only C34 (`scripts/behaviors/live-roster-routing.cucumber.mjs`) after the fix. No broader suite is run.
