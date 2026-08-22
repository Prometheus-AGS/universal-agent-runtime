# Skill scope semantics deterministic verification summary

Profile scope: `server-full` only. These results transfer to no other profile.

- Session selection: PASS. A persisted conversation policy selected a skill for
  one run, then excluded it from the next run before activation and overlay.
- Built-in governance: PASS. Service and HTTP paths refuse built-in edits and
  leave disable available. Installed-pack skills use built-in origin.
- UI behavior: PASS by existing source inspection. Edit and Delete are disabled
  for built-ins while the enable/disable control remains available.
- Durable foundation: PASS through the archived prerequisite's process-reopen,
  scope-precedence, origin, and deletion receipts.
- Negative controls: PASS. Removing either the session filter or immutable-origin
  guard made its focused test fail with exit 101; restored hashes match.
- Tier 0: PASS within the recorded baseline. Check exits 0 with three known
  warnings. Scoped Clippy exits 0 with 572 warnings; no warning-free claim is
  made.
- Tier timing: full phase Tier 2 remains deferred until all active-phase changes
  are complete.

Independent artifact critic: PASS. Independent artifact judge: PASS. All four
blocking constraints are satisfied for this change boundary.
