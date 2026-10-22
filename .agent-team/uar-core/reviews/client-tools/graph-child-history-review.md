# Graph child inherited-history corrective source review

Task 13.24 / CT4, Execute. Artifact-only review of frozen `thread/kernel.rs` inherited-history assignment, `graph/delegation.rs` child-outcome guard, the exact manager trusted-child admission predicate and AgentNode terminal handling. Read `graph-source-run-repair.md`. Only this report was written.

## Disposition

No blocking source finding in this bounded correction. The earlier integrated source review missed the conflict between captured history setting inherited_history=None and manager's unconditional inherited-history admission requirement; this pass explicitly traces that contract.

Captured caller input now deserializes its prepared canonical history into `Vec<Message>` and supplies `Some(history)`. This satisfies manager's existing trusted-child predicate without deleting or weakening its owner, lineage, artifact, session, controls, approval-root, cancellation and bound-MCP checks. Paired tool messages are decoded in their original order without HistoryForkMode filtering or trimming. Malformed canonical history propagates an error. The ordinary branch remains `Some(turn.messages)` after the existing final-user split, preserving its prior behavior. Manager's history precedence selects this canonical inherited history, and caller-aware input append remains disabled, so the generic delegated prompt is not appended again.

`caller_handoff` now returns no handoff for Failed, Cancelled, Yielded or missing child results before requiring a manager run that may never have registered. AgentNode receives the actual outcome and uses its existing explicit error handling; the repair does not convert those outcomes into success. Completed candidates still require the actual child join and Done status, followed by durable exact root/child correlation and visible call validation. Empty Completed output without valid handoff evidence remains an error.

## Evidence and limits

Only `sed`/`cat` source reads and this report write ran. No tests, checks, builds, imports, formatting, compiler probes, runtime or network commands ran. No production file was edited. Global reviewer verification stays CLOSED; the lead owns its scoped local functional loop. Compilation, real provider contact, paired-history continuation, all graph protocols, cold graph restart and cleanup remain unverified for this changed source. No acceptance or publication claim is made. Same-family isolated source review is not cross-model adversarial evidence.
