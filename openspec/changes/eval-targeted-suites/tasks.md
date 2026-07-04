## 1. Fixture providers

- [x] 1.1 `SkillActivationProvider` — real `SkillService`, 5 fixture skills
- [x] 1.2 `RoutingProvider` — real `ProviderRegistry` + `ModelRouter`, 2
      seeded catalog entries, health-cooldown trip support
- [x] 1.3 `ContextEfficiencyProvider` — pure `strategy_for_model` call
- [x] 1.4 `RouteRequirements` gained `Deserialize` for JSON case inputs

## 2. CLI dispatch

- [x] 2.1 `run_suite` recognizes the 3 new suite names and routes to the
      matching fixture provider, bypassing `Orchestrator::new` entirely
- [x] 2.2 All other suite names keep the original real-model path

## 3. Suite files + baselines

- [x] 3.1 `evals/skill-activation.yaml` (6 cases)
- [x] 3.2 `evals/routing-accuracy.yaml` (3 cases)
- [x] 3.3 `evals/context-efficiency.yaml` (3 cases)
- [x] 3.4 Seeded + committed baselines for all 3 (keyless, no operator
      action needed — unlike `starter.yaml`'s baseline)
- [x] 3.5 `.gitignore`: ignore `evals/results/*.json` except
      `*.baseline.json`

## 4. Verify

- [x] 4.1 9 new unit tests in `targeted.rs` (skill match/no-match,
      preferred-provider override, health-cooldown exclusion, impossible
      cost ceiling, small/large context window resolution)
- [x] 4.2 New Tier-1 integration test running all 3 suites through their
      real providers, asserting perfect scores
- [x] 4.3 Live CLI smoke test with no API key anywhere in the shell — all
      3 suites seed baseline, then re-run clean (Δ +0.000)
- [x] 4.4 `cargo test --lib eval::` 44/44 green
- [x] 4.5 Full suite `cargo test --lib` 341/341 green

## 5. Follow-ups (not this change)

- [ ] `starter.yaml`'s own baseline is still unseeded (operator-only,
      carried from `eval-harness-hardening`/`gate-activation-and-security-cleanup`)
      — unrelated to this change's 3 new suites, which ARE seeded.
