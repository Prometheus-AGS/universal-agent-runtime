# B4 deterministic verification summary

Profile scope: `server-full` only. These results transfer to no other profile.

- Scoped precedence: PASS. The service matrix observed conversation over agent over global in both enable-over-disable and disable-over-enable directions.
- Real-run widening and binding: PASS. An embedded SurrealKV run activated a conversation-enabled, globally disabled skill. A disable after binding did not alter that run; the next run emitted no activation and received no overlay.
- Restart and re-registration: PASS after correction. Separate seed, reopen-delete, and verify-deleted child processes opened one SurrealKV directory. The reopened processes observed persisted global and per-agent disables after builtin metadata re-registration.
- Negative controls: PASS. Removing the registration merge produced exit 101 in the cold-restart child. Forcing matching conversation records to enabled produced exit 101 when the next real run activated a disabled skill. Both controls restored to current-candidate source hashes and passed afterward.
- Origin and deletion: PASS. The API serialized `origin: builtin`; builtin deletion returned the existing immutable error and retained the skill. An API-created user skill was removed from SurrealKV and the filesystem and remained absent after another child-process reopen.
- Compatibility: PASS after correction. The pre-existing agent-binding replacement assertions passed for requested IDs that are not currently loaded, while loaded skills also receive durable agent overrides.
- Storage formats: PASS. Global, agent, and conversation records round-tripped through SKILL.md serialization.
- Tier 0: PASS. Package check and package/library/no-deps Clippy exited 0; no warning points at a B4-added line.
- OpenSpec: PASS. `openspec validate skill-scoped-governance --strict` exited 0.
- Scope: PASS. Behavioral changes stay within the operator-amended Track B surface.
- Tier 2: NOT RUN. The phase command remains prohibited until B5 is implemented.

Literal negative-control output is retained in:

- `openspec/changes/skill-scoped-governance/evidence/fail-closed-negative-control.md`
- `openspec/changes/skill-scoped-governance/evidence/live-effect-negative-control.md`

Literal positive output is retained in
`openspec/changes/skill-scoped-governance/evidence/positive-verification.md`.

Uncomfortable result: the first independent review correctly rejected the
same-handle “restart” proof and found that the durable model had silently
dropped the legacy binding API's unknown IDs. Neither issue was visible in the
initial six-test service matrix.
