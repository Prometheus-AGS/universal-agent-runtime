# Tasks

This is a documentation-only handoff. Implementation and all product verification remain unstarted. Tracker: [UAR #519](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/519). The current KBD phase is not reopened. The direct main-spec update already carries the target requirements; reconcile it when eventually archiving rather than duplicating them.

## 1. Establish the implementation phase

- [ ] 1.1 Establish the dedicated KBD phase, single-writer ownership and bounded authorization; deliver the recorded phase ledger before any production change.
- [ ] 1.2 Resolve exact native adapter/SDK pins and supported device/capability matrix, including system-model 26 availability versus custom-provider 27 and Core AI 27.2 presets; deliver source references and an admission table.

## 2. Bootstrap and trusted setup

- [ ] 2.1 Implement system-model availability and first-run selection through UAR routing; preserve explicit settings and opt-outs. Author production-path scenarios and updated configuration docs; acceptance at the authorized gate must show local assistance or an actionable manual fallback.
- [ ] 2.2 Implement durable setup state/consent and its entity projection, cancellation/retry/resume, credential boundary and atomic provider activation. Author interrupted/denied/uncertain-effect scenarios and operator docs; acceptance must show no duplicate effect or unapproved transition.

## 3. Optional provider and embedding boundaries

- [ ] 3.1 Implement the selected MLX, Core AI or flint-loader bridge with explicit capabilities, loader leases and a single UAR tool loop. Author unsupported-format/hardware and single-dispatch scenarios plus adapter docs; acceptance must trace every tool effect to UAR and every custom-model lifecycle operation to flint-loader.
- [ ] 3.2 Preserve independent generation, knowledge and memory registry bindings. Author generation-switch and explicit reindex scenarios plus migration docs; acceptance must show unchanged embedding identity/dimensions/indexes unless a separate migration is approved.

## 4. Full-phase VLBV evidence

- [ ] 4.1 Reconcile all production implementation items and record the exact completed source inventory; product verification stays closed until every implementation item is complete and the required grant exists.
- [ ] 4.2 Execute only the authorized real-path integration gate specified in design.md; retain exact commands, source/OS/model identities, redacted routing/consent/egress/cancellation evidence and failures without upgrading earlier receipts.
- [ ] 4.3 At the device/release boundary, qualify actual supported Mac and any claimed mobile profiles, migration and rollback; retain device evidence and independent artifact review before support or performance claims.

## Workflow follow-up

- Archive only after the authorized implementation and evidence are complete; preserve the proposed-versus-verified distinction until then.
