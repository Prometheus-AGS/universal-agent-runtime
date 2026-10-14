# Dot17 calendar source handoff

Phase: Dot Execute. Status: source authored, no verification executed.

Owned changes are `scripts/dot/calendar_journey.py` and
`docs/dot-calendar-qualification.md`. The runner owner integrates the exact
`run_calendar(config, evidence)` entrypoint as a separate stage; it owns its
real child-process runtime(s) and controlled HTTP provider. No arbitrary callback,
precomputed pass receipt, runtime test clock, or protected-store write is used.
The documentation contains the complete profile and guest-clock configuration.

The real guest clock advances through the fixed installed calendar cases, then
an approval-held one-shot, unknown-effect daily coalescing/revision/restart hold,
and paused latest-due catch-up. The actual host scanner admits actual logical
instance commands. A real MCP peer records effects independently and disconnects
after committing an unknown effect. That uncertain predecessor remains held.
Two shared runtimes read the same durable schedule state; the driver never
claims process restart proves supervisor exclusion.

The uncomfortable case is VM time left advanced after abrupt runner death or
unconfirmed child cleanup. The driver checks dedicated guest identity, rejects
containers and physical hosts, binds provisioning evidence, observes clock drift,
and restores the initial time plus elapsed monotonic time in normal cleanup.
The disposable VM remains the isolation boundary. Guest credentials, deployment
grants and certificates must span the complete 2027 fixture interval.

Source integration remains the runner owner's responsibility. Actual VM
provisioning and real installed binaries/configuration are gate prerequisites,
not supplied passing results. All Python syntax, runtime API composition, model
counts, effect counts, DST behavior, restart, host cleanup and restored clocks
remain unverified under the full-phase boundary. No tests, checks, builds,
formatters, validators, compiler probes, scripts or review gates were executed.
No unrelated production change or canonical task mutation was added.

Shared-profile clock prerequisite corrected with runner approval: its actual
durable SurrealDB must resolve to guest loopback and share this VM clock domain.
The driver requires digest-bound database provisioning evidence; a remote clock
cannot qualify database `time::now()` invariants merely because runtime clocks
were advanced. No provisioning or clock operation has been executed.
