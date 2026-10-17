# V2 local07 repair handoff

Phase: `uar-outstanding-issues-v1-1-1`, Execute, plan revision63. R11 was
implemented and L9 inventory10 sealed before local07. The six-group local07
command exited1. Its immutable receipt is
`evidence/acceptance/20261017-local-07/receipt.json`, SHA-256
`799b46fe6bfc6155f01150c05ed1818adcf7b3690b0955fdb4103b57e2acea45`.
The installed arm64 image ID was
`sha256:2fdcc9b3520b28f8d7daa1cdb0c324f66f45e1bb71ac5a75dfa63d5fce8a7cf1`.
The private `qualified-rc07-acceptance-identity-02.json` receipt sealed an
actual isolated guest reboot, same installed executable, live predecessor
queued/running attempts and distinct replacement fence. Recovery did not reach
that proof, so it was not consumed.

| Group | Local07 | Observed cause | Owned repair |
| --- | --- | --- | --- |
| binding | failed | The enabled and allowed fallback case made three configured resilience attempts on primary 503 before one authorized fallback; its fixture asserted one primary attempt. | `uar-providers` changed only `scripts/release/uar_1_1_1_runtime.py` to disable outer resilience retries in that scenario. |
| streams | failed | The graph scenario selected `spawn_agent` but production graph delegation requires `spawn_agent` and `wait_agents` for the host-owned child join. The 4,233-case wire partition matrix and later tool-loop artifacts completed first. | `uar-runtime` diagnosis; `uar-providers` added `wait_agents` to that fixture artifact in the same runtime script. |
| mcp-outcomes | passed | Connected MCP descriptor and typed receipt path exercised on the installed image. | Preserve receipt. |
| credentials | failed | The correction scenario called `/api/chat/completion` before model verification and got HTTP422. The runner copied this first exception to unfinished credential scenarios; those local, Surreal and Postgres cases did not execute. | `uar-trust-tools` added explicit host/admin provider model verification before the scenario's provider-contact baseline in `scripts/release/uar_1_1_1_credentials.py`. |
| recovery | blocked | The embedding peer held startup built-in skill requests on `release`; UAR connected to healthy Surreal but never reached health before the 180-second wait. | `uar-state` changed only `scripts/release/uar_1_1_1_recovery.py` so normal startup batches receive vectors immediately and only the unique uploaded recovery source is held. |
| combined | passed | Existing combined installed-image observation completed. | Preserve receipt. |

The four source corrections were made after local07 terminated. No test,
compiler check, build or new acceptance gate ran during authoring. The next
lead action is to refresh every whole-phase inventory row and the source and
fixture trees, then record an explicit complete inventory before any affected
gate. Use local07 as the immutable repair parent and run only its failed or
blocked groups. Since changed scripts participate in source and fixture
identity, establish a truthful same-image candidate/guest identity for the new
hashes before recovery. Do not reuse an old identity receipt with altered
bytes. The #387 two-principal compiler adapter remains unverified until the
credentials group reaches it. Published arm64/amd64 child gates, release and
issue receipts remain pending.

Uncomfortable fact: two local07 groups passed, but the phase has no accepted
local program. Treating the failed credentials result as coverage for the
unexecuted compiler and migration cases would hide the release blocker.
