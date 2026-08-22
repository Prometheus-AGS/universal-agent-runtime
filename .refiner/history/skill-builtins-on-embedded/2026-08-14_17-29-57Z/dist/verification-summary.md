# B3 deterministic verification summary

Profile scope: `server-full` only. These results transfer to no other profile.

- Fresh embedded catalogue: PASS. A fresh temporary SurrealKV database produced every built-in returned by `discover_builtin_skills()` in the runtime registry.
- Persistence and restart: PASS. All built-ins were readable directly from persistence before the second runtime construction; every built-in then appeared exactly once in the registry and database.
- Seeding switch: PASS. `seed_defaults(false)` produced none of the discovered built-ins in the registry or database.
- Negative control: PASS. Removing the embedded registration block produced exit 101 at `fresh embedded registry is missing builtin builtin::upload-to-bossfang`; the pre/post restoration diff hash matched and the restored assertion exited 0.
- Tier 0: PASS. Package check and package/library/no-deps Clippy exited 0 with no B3-owned warning.
- OpenSpec: PASS. `openspec validate skill-builtins-on-embedded --strict` exited 0.
- Scope: PASS. Production behavior changed only in embedded bootstrap; persistence and scoped governance were not changed; `docs/SPECIFICATION.md` changed only its sanctioned GAP-05 line.
- Tier 2: NOT RUN. The phase command remains prohibited until B4 and B5 are implemented.

Literal negative-control output is retained in
`openspec/changes/skill-builtins-on-embedded/evidence/fail-closed-negative-control.md`.
