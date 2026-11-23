# Re-review of diff-r2 (2e971bbf): BLOCK; steps PASS, runner REJECT

| # | Severity | Finding | Triage |
|---|---|---|---|
| 1 | HIGH | Local-owner chat needs `security.provider_credentials.installation_id`; the empty config guaranteed none, so every turn fails `credential_source_denied` (policy.rs:125-143, owner.rs:18, docs/security/jwt-validation.md:162-163) | FIXED: synthetic id in the scratch config. Lead confirmed in source |
| 2 | MEDIUM | diag-up.sh and probe-server.sh still lack CONFIG_FILE | FIXED: both retired (exit 64) |
| 3 | MEDIUM | The baseline check cannot detect harness faults | LIMIT: run one chat scenario straight after prepare-bdd, before the mutant builds (put in the amendment) |
| 4 | LOW | Skill directories and parent environment are not hermetic | LIMIT, no paid route found |
| 5 | LOW | Retained focus key can create a distant tab stop | ACCEPT |
| 6 | LOW | Browse mode skips the hidden description | FIXED: sr-only paragraph before the list |
| 7 | LOW | Scratch config written per process, never removed | ACCEPT |
| 8 | LOW | static/ is dirty | Commit named files only |
