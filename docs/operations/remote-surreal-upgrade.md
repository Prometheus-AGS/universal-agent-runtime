# Remote SurrealDB upgrade: binding, budgets and knowledge

This procedure covers the failures reported in
[#385](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/385).
The operator subsequently confirmed that v1.1.0 serves the site after enabling
durability attestation and changing its token policy; the #384 inherited timeout
fix then allowed 141–187 second responses. That report establishes the operator's
outcome. It does not certify a new release or another deployment.

## What changed for an older deployment

The durable accounting, upload recovery and principal-isolation contracts landed
with the runtime support work in `22088cee`, between `3e6af7c4` and the #384 merge.
The #384 timeout/retry inheritance fix is retained. Its presence in both failing
images did not make it the cause of their accounting admission failures.

| Symptom | Required interpretation and action |
| --- | --- |
| Immediate model-binding refusal on a budgeted agent | A configured token or cost envelope requires eligible durable accounting and a verified principal. An unattested remote Surreal endpoint is ineligible. Establish actual backing and recovery guarantees before the operator sets attestation. |
| Upload returns 503; dispatcher disabled | The same remote durability prerequisite applies before upload may commit source bytes, document and job or return 202. It is separate from provider credentials and model verification. |
| Budget admission still refuses after attestation | Without a proven final-wire request budget contract, each call reserves catalog context plus maximum output. A 16,000 total-token ceiling cannot admit a larger reservation even when the prompt is small. Inspect the safe reservation diagnostic and choose an explicit adequate envelope for the supported request path. |
| A different, empty authenticated KB appears | Authenticated knowledge now occupies the complete verified principal's separate tables. Legacy rows remain retained and are not automatically adopted. Reupload retained originals under the current verified identity; update the agent to that new KB. |
| Team catalog starts read-only with TEAM_EXECUTION_OWNER_CONFLICT | A prior incarnation still owns the durable execution claim. Single replica, elapsed time or a missing Pod is not exclusion proof. Use exact joined quiesce/reclaim or qualified supervisor exclusion. |

## Remote persistence configuration

Keep the selected remote URL, namespace/database, verifier issuer/audience and
embedding settings stable. Retain the database's real persistent backing and
document a recoverable operational procedure. After the responsible operator
establishes those guarantees, its trusted launch environment may set:

```text
UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1
```

The provider captures eligibility at construction; changing an environment value
does not change an existing instance. Apply configuration through the operator's
normal controlled restart/rollout. A successful connection, retained image,
replica count or storage volume name alone does not establish durability.
No fixture attests know-me storage or changes its configuration.

The runtime warns at startup when persisted token/cost-limited agents are paired
with storage that cannot support durable accounting. The warning reports a safe
category, eligibility and agent count. Run admission remains authoritative for
agents added later, request overrides and current scope limits.

## Preserve deliberate token and cost policy

The customer's removal of its 16,000-token limit was an operator workaround;
this release does not remove, widen or disable explicit limits. Token ceilings
cover input plus output. Provider-reported actual usage and conservative admission
reservations are different quantities. An explicit request `max_tokens` is not
alone proof of a smaller complete input/output reservation. Smaller reservations
require the supported exact final-wire contract, including reasoning and
uncertainty allowance; otherwise catalog maxima remain authoritative.

An ERROR diagnostic at the per-call refusal reports `model_call_admission`,
`budget_admission_failed`, a safe accounting cause and diagnostic reference,
reservation source/bound and configured turn/session token limits. Binding
refusals separately report their safe stage/category/reference. Neither prints
provider URLs, credentials, request text or raw storage/constructor error chains.
Use the run ID to correlate these records. Persisted scope limits and outstanding
liability survive restart; configuration edits do not erase them.
See [durable accounting](durable-accounting.md) for explicit scope changes and
reconciliation. Do not clear accounting rows to make a run pass.

## Model verification and KB recovery are separate

Model re-verification cannot repair missing durability attestation, an
unaffordable reservation or a legacy ownership boundary. There is no blanket
requirement to erase existing verification receipts merely because the runtime
image changes. Target-selection workflows require a current receipt bound to
the actual provider configuration; changed or absent configuration proof needs
the existing authorized provider-test operation. Direct per-agent model capture
still enforces route, credential and budget authority. Keep `llm.timeout_secs=240`
and deliberate retry settings when selecting a per-agent catalog provider.

No automatic authenticated KB migration is supplied. Preserve legacy tables and
backups. With the current verified issuer, tenant and subject, create/reuse a
principal KB and reupload original bytes through the authenticated document API.
Wait for the durable job to index, retrieve as the same owner and update the
agent's explicit KB reference. Old IDs, matching subject strings and other owners
cannot adopt the old chunks. Missing originals remain a recovery limitation.
See [knowledge ownership](knowledge-principal-isolation.md) and
[durable ingestion](durable-ingestion.md).

For the execution-owner warning, inspect the current claim before following
[supervisor recovery](supervisor-recovery.md). A previous runtime may quiesce
and join its roots before exact reclaim. An abruptly lost runtime needs genuine
qualified exclusion evidence; deleting its claim or resetting its epoch is not
a recovery procedure. Catalog conflict does not authorize takeover or establish
that ordinary chat or provider verification failed.

The uncomfortable case is a correctly refusing upgraded runtime whose previous
deployment relied on unenforced policy or legacy identity. Diagnostics and safe
recovery instructions cannot manufacture durability, missing source bytes,
affordable budget capacity or physical exclusion proof.
