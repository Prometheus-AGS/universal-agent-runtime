# #463/#464 mounted capability fixture plan

2026-10-31. SOURCE INSPECTION AND DRAFT ONLY. Verification lock CLOSED for this
assignment. No server, proxy, model call, test, check, compiler, build or external
mutation was started. The shared checkout remains frozen. Only this `/tmp` file
was written. Command forms below were read from source; availability/readiness
of binaries, dependencies, listeners and models was not probed.

## Existing runner and identity boundary

`scripts/behaviors/README.md:13–15,30–38` defines the HTTP-only manual Cucumber
runner and external identity boundary. Its existing invocation is:

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/cucumber.mjs
```

That config selects only streaming.feature. Existing bounded reference configs
use the same runner with `--config scripts/behaviors/structured-output.cucumber.mjs`,
`chat-stream-modes.cucumber.mjs`, or `native-run-streams.cucumber.mjs`. They set
`retry:0`, explicit feature/step paths, progress output and a JSON report selected
by optional `UAR_CUCUMBER_REPORT`. Node >=20 and installed Cucumber are documented;
do not invoke the global BDD/package runner that builds or starts other suites.

Nonsecret inputs are `UAR_BASE_URL` (credential-free externally provisioned
identity endpoint), `UAR_WORKSPACE_ID` (ownership selector), `UAR_MODEL`
(`provider/model`), and `UAR_CUCUMBER_REPORT` (new output path). Existing E05 uses
`UAR_STRUCTURED_MODEL`. Scenarios send Content-Type, workspace selector and SSE
Accept where needed, never Authorization. No authentication creation, copying,
renewal or forwarding belongs in new manual steps.

The historical C54 receipt records client endpoint `http://127.0.0.1:60675` and
nonsecret workspace/model selectors. This establishes historical configuration
only, not a currently available listener. README's default port 3000 is likewise
an example. No current endpoint availability was established; lead/external
provisioning supplies the owned identity endpoint for the next phase. No private
environment/config or secret values were read or printed in this investigation.

## Available deterministic peers

| Existing source | Invocation / relevant contract | Reuse / limitation |
|---|---|---|
| `src/bin/stub-llm.rs:1–9,78–106`; `tests/bdd/playwright.config.ts:54–71` | Existing prebuilt CLI form: `STUB_LLM_PORT=4600 target/debug/stub-llm scripts/acceptance/fixtures/dedicated-runtime-stub-llm.json`. CLI accepts argv fixture or STUB_LLM_FIXTURES, binds loopback, defaults port4600. | Deterministic keyless OpenAI-compatible chat/stream/tools; use already-owned separately provisioned binary only after lead opens exact gate. Presence/compatibility of executable not probed. |
| `tests/integration/live/stub_llm.rs:94–106,186–205,246–267` | `/v1/models`, `/v1/chat/completions`, GET/POST `/_stub/requests`; captures raw request JSON in order and reset clears it. Fingerprint uses model, last user string, tools present and tool-result present. | Captured request proves wire delivery. Array-valued image content currently fingerprints as empty string; use explicit fixtures or a narrowly added content-aware match. Never claim canned response demonstrates vision inference. Request capture must remain isolated and synthetic; don't export raw image bytes or headers into general diagnostics. |
| `scripts/post_lifecycle/controlled_peer.py:95–112,315–324` | Existing parser supports `python3 scripts/post_lifecycle/controlled_peer.py --port 0 --port-file <fresh-path> --receipt-file <fresh-path> --dimensions 384`; defaults bounded hold20s. Supports `/v1/chat/completions` and embeddings, body-free counters/digests, case controls, deterministic failure/disconnect/tool modes. | Source describes it as unexecuted fixture, not provider-exact oracle. Useful for routing no-dispatch and failure counts but currently no image semantic oracle. Do not resurrect its historical broad boundary runner. |
| `scripts/behaviors/support/provider-fault-proxy.mjs:1–8,11–16,53` | `UPSTREAM=http://127.0.0.1:4000 node scripts/behaviors/support/provider-fault-proxy.mjs`; emits PROXY_URL on dynamic loopback port. `/__fault/arm`, `/clear`, `/log` cut streams and record counts/status. | Could exercise fallback; existing implementation forwards headers unchanged and targets HTTP upstream. Keep any use confined to externally provisioned synthetic peer, never credential-bearing live gateway merely for this phase. |

Do not reuse `startDedicatedRuntime` from
`scripts/acceptance/fixtures/dedicated-runtime.mjs:143–220` in new manual steps.
It launches Docker databases/UAR/proxy and generates/passes signing material.
Its provisioning/authentication responsibilities conflict with the current
manual-test boundary. External host provisioning remains outside scenarios.
Do not use Playwright's webServer configuration as the mounted API runner:
its setup spans additional servers/build prerequisites and is unnecessary here.

## Minimal next-phase additions (proposed paths, not existing commands)

1. Add `scripts/behaviors/features/capability-dispatch.feature`, matching
   `capability-dispatch.cucumber.mjs`, and
   `steps/capability-dispatch.steps.mjs`. Select only #463/#464 cases with retry0;
   reuse HTTP/evidence/cleanup conventions from structured-output and native-run
   stream steps. Keep actual requests through public mounted UAR adapters and
   external identity endpoint. Do not assert solely against `/api/uar/route`.
2. Reuse existing stub JSON fixture schema for deterministic two-destination
   responses, request capture and planned fallback failure. If needed extend
   the existing shared stub with only image-aware request matching and a bounded
   scripted failure response. The CLI currently supports content,
   grounded_content and tool_call only (`src/bin/stub-llm.rs:37–53`), so failure
   or image assertions are explicit proposed additions, not existing features.
3. Host provisions isolated model/provider registrations and completes their
   existing operational verification before manual scenario baseline; no real
   provider settings or shared live catalog should be repointed. Actual config
   fields are `ProviderConfig` and `ModelConfig` in `src/llm/registry.rs:33–115`:
   id/base_url/protocol/default_model/models/enabled, context_window,
   supports_vision/tools/structured_output/streaming, max_output_tokens. API key
   is optional; keyless peer must not turn a credential placeholder into an auth
   scenario. Existing administration paths are `POST /api/uar/providers` and
   `POST /api/uar/providers/{id}/test` (`providers.rs:41,199–238,404–435`). These
   actions are provisioning/verification, not authorized by this plan.
4. #463 deterministic transport cases: mixed text/image and multiple images
   retain ordering/identity through each selected adapter; compare synthetic
   bytes/digests at captured provider boundary. Streaming/nonstreaming supported
   forms plus malformed/oversized/unsupported model/form cases must refuse
   before peer call. Do not add URL fetching to satisfy fixtures. A separate
   selected real-vision-model image with fresh private visual fact is required
   for actual vision credit; deterministic canned text is only wiring evidence.
5. #464 deterministic dispatch cases: at least two explicitly configured
   destinations with different capabilities/window; verify actual selected
   peer/model and zero calls to excluded/disabled/ineligible routes. Force an
   eligible fallback and smaller window, require per-destination final budget
   preservation (#388); no eligible destination must fail with zero provider
   calls. Separate provisioning probes from case counters. Constraint request
   fields must follow the implemented authoritative API contract; current
   manager `:6502–6504` uses preferred-provider plus default requirements, so do
   not invent requirement/exclusion input keys or certify query-only routing.

Existing `structured-output.steps.mjs:11–35,44–64,117–132` demonstrates disposable
agent creation, no-tools placeholder (empty allow means Auto), fresh session,
policy-bound provider/model, raw exchanges and deletion of only owned agent.
`context-budget.steps.mjs:12–26` reconstructs nonsecret provider view without
api_key. Existing provider PUT invalidates all model verification receipts; the
gotcha and `rest-model-restart.steps.mjs:159–163,290` require preserving original
verified set. Prefer isolated providers to avoid that shared-state restoration
problem. Keep cleanup failures as failures; no retry that hides first outcomes.

Future runner form, only after files exist and lead authorizes execution:

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/capability-dispatch.cucumber.mjs
```

This is a proposed invocation, not an executed or currently available scenario.
Existing steps hardcode detailed evidence under `.prometheus/verification/`;
`UAR_CUCUMBER_REPORT` changes the Cucumber report only. Therefore no scenario may
run against this frozen checkout. Run only after implementation freeze is
released and a new exact candidate/evidence location is selected.

## Unresolved prerequisites

- Lead selects supported image adapters/forms and authoritative dispatch
  constraint contract before steps name input fields. Remaining routes need
  explicit unsupported outcomes, not silent text-only success.
- External provisioning supplies owned identity endpoint and at least two
  isolated keyless deterministic provider/model registrations, with same mounted
  runtime/artifact and known peer ownership. Current listener availability is
  unverified; manual tests never solve it with authentication plumbing.
- Lead selects reusable prebuilt peer versus minimal shared-stub extension,
  optional real-vision model/budget, and approved serial gate commands after
  implementation. No build/output ownership is changed by this plan.
- A deterministic transport/routing PASS cannot close genuine visual inference,
  every provider/protocol, installed-platform or full release qualification.

No speculative production implementation or new global runner is proposed.
