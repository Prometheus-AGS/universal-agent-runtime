## 1. Metrics primitives (shipped prior turn)

- [x] 1.1 `record_skill_activation`/`record_skill_activation_outcome`
      (`src/uar/telemetry/metrics.rs`).

## 2. Wire into the matching path (this pass)

- [x] 2.1 `SkillService::match_skills` records an activation decision per
      matched skill, labeled by the backend actually used.

## 3. Verify

- [x] 3.1 `cargo check --lib` green.
- [x] 3.2 Full-suite batch checkpoint: 318/318 lib tests green (no new unit
      tests added — the change is a metrics side-effect on an existing,
      already-tested code path; `match_skills`' own behavior is unchanged).

## 4. Follow-ups (disclosed, not this pass)

- [ ] `record_skill_activation_outcome` wiring — requires correlating
      activation decisions against the run's tool-call stream.
- [ ] Candidate-vs-considered-but-rejected visibility (true precision, not
      just recall) — the matching algorithms don't currently expose this.
- [ ] Dedicated API/console surface for the counters (currently only via
      the generic `/metrics` Prometheus endpoint).
