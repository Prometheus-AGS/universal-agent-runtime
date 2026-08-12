## 0. Read first

- [ ] 0.1 Read `.kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md`.
- [ ] 0.2 This change removes data. Every task below is written to be reversible.
      If a step cannot be made reversible, stop rather than proceeding.

## 1. Tombstone marker

- [ ] 1.1 Add a durable removed/tombstoned marker to the skill record. Do NOT
      reuse `enabled` — an operator disable and a config removal must remain
      distinguishable, or restore cannot know what to restore.
- [ ] 1.2 Exclude tombstoned skills from matching and from default listings;
      keep them retrievable for restore and audit.

## 2. Reconciliation pass

- [ ] 2.1 After providers load at startup, compare the configuration source
      against stored skills.
- [ ] 2.2 Upsert skills present in configuration.
- [ ] 2.3 Tombstone stored skills that are config-provisioned
      (`provider_id == "fs-skills"`, `server.rs:472-476`) and absent from config.
      **Match on `provider_id`; do not infer the source any other way.**
- [ ] 2.4 Clear the tombstone when a skill reappears in configuration, preserving
      its scoped configuration from `skill-scoped-governance`.
- [ ] 2.5 Log every tombstone at info with skill id and reason.

## 3. Fail-safe

- [ ] 3.1 If the configuration source yields zero skills while the database holds
      config-provisioned skills, tombstone nothing and log at error level.

## 4. Proof

- [ ] 4.1 Add / change / remove / restore round trip through restart.
- [ ] 4.2 API-created skill survives an empty configuration source.
- [ ] 4.3 Built-in survives reconciliation.
- [ ] 4.4 Empty-source fail-safe tombstones nothing.
- [ ] 4.5 Tombstoned skill is excluded from matching.
- [ ] 4.6 Restore preserves scoped configuration.
- [ ] 4.7 **Negative control** for 4.4: remove the fail-safe in a scratch build
      and show the test fails. Record command and failing output.
- [ ] 4.8 **Negative control** for 4.2: make reconciliation ignore `provider_id`
      and show the API-skill test fails. This is the guard that prevents data
      loss; prove it works.

## 5. Stop conditions

- [ ] 5.1 A task appears to require hard-deleting any skill → stop. Operator
      decision 2026-08-12 is tombstone-with-restore.
- [ ] 5.2 `provider_id` turns out not to distinguish config-provisioned from
      user-created skills in some path → stop and report. The whole safety
      argument rests on it.
- [ ] 5.3 Reconciliation appears to need to touch agents, knowledge bases, or
      providers → stop; skills only.
- [ ] 5.4 A pre-existing unrelated failure appears → stop and report.
