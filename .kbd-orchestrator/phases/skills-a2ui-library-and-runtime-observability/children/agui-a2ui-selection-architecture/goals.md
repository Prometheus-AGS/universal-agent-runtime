# Presentation phase goals

Date: 2026-09-04. Scope: the three existing Presentation changes, not a new release phase.

1. Operators can create, edit, preview, disable and delete reusable, owner-scoped UI templates in a production Presentation workspace. Templates survive a restart with the selected persistent backend. The development-only A2UI tester stays development-only.
2. UAR resolves Presentation eligibility through global, agent, conversation and turn policy without widening a parent restriction. Client rendering support further restricts output, never grants resource access.
3. Each run records requested and effective text/A2UI/hybrid behavior, the reason for fallback and any rendered template identity/revision. Selection is not a claim of delivery. Existing clients retain their current behavior when negotiation is absent.

Exit: local phase-boundary tests establish persistence/ownership, validation, non-widening policy, legacy compatibility, observable selection and the production UI workflow. Inspect desktop and narrow-screen captures and complete independent UI/implementation review. No product testing in GitHub Actions.

The uncomfortable constraint: the ledger's two completed prerequisite rows have observed code gaps. Corrective tasks must supply the missing implementation; the recorded count does not waive these goals. Eight other outstanding ledger rows concern four release-tail changes cancelled by a later operator decision. Do not reinstate or mark those gates passed to manufacture 120/120.
