# Illustrative collaboration examples

Status: documentation fixtures for the official draft. These examples do not install agents, issue commands or publish external content. Marketing/design execution and executive representation are later roadmap items; examples define interoperable authoring shape only.

| Directory / file | Story | Schema |
|---|---|---|
| development/ | Coordinator, two parallel implementers, bounded design subteam and delayed integration role | kind-selected definition schemas plus package-manifest |
| product-feedback/ | Intake, product proposal, independent critic, exact-effect operator approval and issue creation | kind-selected definition schemas plus package-manifest |
| marketing/ | Research, positioning, copy and critique; no publication | kind-selected definition schemas plus package-manifest |
| design/ | Brand/logo, mobile and accessibility collaboration | kind-selected definition schemas plus package-manifest |
| deployment-binding.json | Explicitly inactive installed-binding example using references, not secrets | deployment-binding |
| runtime-snapshot.json | Idle team with one ready task and durable queued message | runtime-snapshot |
| surface-action.json | View-task action, with untrusted identity hints and expected surface revision | surface-action |

Each package contains immutable references and a lock with real calculated content/file digests. Do not mutate a referenced file without recalculating both layers and its dependents. Definition version 1.0.0 is an illustrative package-local version, not a release claim about any executable dependency. `skills:[]` intentionally avoids inventing executable skill artifacts; future full/mini exports MUST bind actual installed skill versions/digests and retain required/config values. The inactive binding's example provider/model/storage references are deliberately nonoperational and must be resolved by a fresh installation; it is not a runnable configuration.

Workflow inputMapping values use `workflow-input:<field>` or `artifact:<task-id>:<field>` references. They select authorized data; they do not execute expressions or expand access. All declared dependencies must settle successfully before a dependent task becomes ready, even when only one predecessor's artifact is mapped into its brief.

Development implements its entire bounded feature set before the integration task. The product-feedback approval-request step does not mint authorization: the publisher's actual effect must be approved with its actor, destination, payload and effect identity at the trusted boundary. An uncertain issue-create result is reconciled before retry; maxAttempts=1 avoids implying a safe automatic retry.

Use the separate protocol trace fixtures to inspect streaming, surface attribution and recovery. Fixtures are not recorded runtime evidence. The complete artifact gate checks schema/example agreement once; later implementation gates prove behavior.
