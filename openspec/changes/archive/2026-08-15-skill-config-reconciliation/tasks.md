## 0. Read first

- [x] 0.1 Read `.kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md`.
- [x] 0.2 This change removes data. Every task below is written to be reversible.
      If a step cannot be made reversible, stop rather than proceeding.
- [x] 0.3 Correct the observed provenance defect before reconciliation: a cold
      filesystem reload SHALL assign `provider_id == "api"` beneath the reserved
      `skills/dynamic/` directory and `provider_id == "fs-skills"` elsewhere.
      Only API-managed skills may be written into that reserved directory. Prove
      both sides with a focused cold-reload test.

## 1. Tombstone marker

- [x] 1.1 Add a durable removed/tombstoned marker to the skill record. Do NOT
      reuse `enabled` — an operator disable and a config removal must remain
      distinguishable, or restore cannot know what to restore.
- [x] 1.2 Exclude tombstoned skills from matching and from default listings;
      keep them retrievable for restore and audit.

## 2. Reconciliation pass

- [x] 2.1 After providers load at startup, compare the configuration source
      against stored skills.
- [x] 2.2 Upsert skills present in configuration.
- [x] 2.3 Tombstone stored skills that are config-provisioned
      (`provider_id == "fs-skills"`, `server.rs:472-476`) and absent from config.
      **Match on `provider_id`; do not infer the source any other way.**
- [x] 2.4 Clear the tombstone when a skill reappears in configuration, preserving
      its scoped configuration from `skill-scoped-governance`.
- [x] 2.5 Log every tombstone at info with skill id and reason.

## 3. Fail-safe

- [x] 3.1 If the configuration source yields zero skills while the database holds
      config-provisioned skills, tombstone nothing and log at error level.

## 4. Proof

- [x] 4.1 Add / change / remove / restore round trip through restart.
- [x] 4.2 API-created skill survives an empty configuration source.
- [x] 4.3 Built-in survives reconciliation.
- [x] 4.4 Empty-source fail-safe tombstones nothing.
- [x] 4.5 Tombstoned skill is excluded from matching.
- [x] 4.6 Restore preserves scoped configuration.
- [x] 4.7 **Negative control** for 4.4: remove the fail-safe in a scratch build
      and show the test fails. Record command and failing output.
- [x] 4.8 **Negative control** for 4.2: make reconciliation ignore `provider_id`
      and show the API-skill test fails. This is the guard that prevents data
      loss; prove it works.

## 5. Stop conditions

- [ ] 5.1 A task appears to require hard-deleting any skill → stop. Operator
      decision 2026-08-12 is tombstone-with-restore.
- [ ] 5.2 `provider_id` turns out not to distinguish config-provisioned from
      user-created skills in some path → stop and report. The whole safety
      argument rests on it. The 2026-08-15 `skills/dynamic/` cold-reload defect
      is the sole operator-approved repair; any additional path still stops.
- [ ] 5.3 Reconciliation appears to need to touch agents, knowledge bases, or
      providers → stop; skills only.
- [ ] 5.4 A pre-existing unrelated failure appears → stop and report.
