# Execute — planning reconciliation

This child implements planning artifacts and canonical scheduling only. It adds OpenSpec requirements for all nine issue379 items, reconciles existing tasks with the full-phase boundary, and registers the combined parent work. It does not implement or certify production behavior.

Ownership: dot_parent_plan writes new OpenSpec18–20; surface_architecture_audit reconciles existing task documents; root owns parent plan, sequencing manifest, canonical mutations, reflection/handoff. GPT-6 Astra inspected canonical API semantics read-only. All share this checkout and preserve unrelated changes.

Existing task registration is immutable and has no task kind or dependency field. Preserve all historical completion and all deferred verification obligations. Record revised effective task descriptions and implementation prerequisites in a versioned sequencing manifest and immutable KBD decision. Use explicit source-task transitions; never run a test to unlock the next production task. Dot17 gains backend3 for runner/fixtures/docs implementation; backend1/2 stay post-boundary acceptance/publication. No product tests, validators or builds run in this child.
