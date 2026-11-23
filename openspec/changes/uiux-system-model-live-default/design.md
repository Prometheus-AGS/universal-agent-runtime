## Context

Operator decisions of 2026-11-19 (read in full in `.prometheus/decisions.md`): the system
level is the registry default provider plus its `default_model`, set through the live
route; an agent's explicit model always beats it; `llm.model` leaves the operator UI;
`run_policy.global.model` is retired from model selection or kept internal, never a second
control; the picker must be reachable from primary navigation. Build items 12, 13, 16.

Verified source facts (read at the cited lines in this session; nothing was run):

| Fact | Source |
|---|---|
| The live route persists then swaps; 404 for an unknown provider; 500 on persist failure, live default unchanged | `src/uar/api/providers.rs:367-399` |
| Route mounted at `/api/uar/providers` and the alias `/api/providers` | `src/server.rs:2532`, `:2680` |
| `set_default_provider_id` writes the `llm.default_provider` row | `src/uar/settings/manager.rs:1247-1249` |
| Boot hydrates the registry default from that row once | `src/uar/settings/provider_sync.rs:21-23` |
| Registry default pair requires an enabled default provider with a non-empty `default_model` | `src/llm/registry.rs:551-563` |
| The model half is written by `PUT /providers/{id}`; a stale execution-profile revision returns `TEAM_REVISION_CONFLICT` as 409; an omitted API key is preserved | `src/uar/api/providers.rs:271-297` (409 mapping at :285); `src/llm/registry.rs:161-192` |
| Resolver order Global, Agent, Conversation, Turn; Global `model` applied at :538 | `src/uar/domain/policy.rs:510-566` |
| Agent scope model is the artifact default when provider and model are both non-empty, overridden by the `uar.run_policy` extension model | `src/uar/domain/policy.rs:262-285`, `:304-305` |
| Registry default fills the route only when the resolved route is empty; falls back to YAML `llm_config.model` when the registry has no pair | `src/uar/runtime/manager.rs:11323-11356` |
| Child model: extension model, then child artifact model, then parent effective model; an unrouted provider yields `CredentialNotGranted` | `src/uar/runtime/thread/policy_intersection.rs:582-590`, `:598-611` |
| Built-in agents seed an empty default and defer to the registry | `src/uar/defaults.rs:21-36` |
| Frontend: `setDefaultProvider` posts the route; `updateProvider` PUTs the full config | `frontend/src/features/providers/api/providers-api.ts:57-73` |
| Frontend: `setDefault` optimistically writes `default_model: null` into `ProviderMeta` | `frontend/src/features/providers/model/providers-store.ts:156-176` |
| Frontend: Models-page star writes only the provider's `default_model` | `frontend/src/features/models/model/models-store.ts:111-121` |
| Frontend: Providers is a primary destination; Models is not; palette lists only nav destinations | `frontend/src/app/shell/nav-destinations.ts:89-96`, `command-palette.tsx:40` |
| Frontend: agent "configured" test ignores the extension model | `frontend/src/features/agents/ui/agents-page.tsx:206-225` |
| Visual authority shows a "Default chat" role card with a Change action | `docs/design/models-and-providers.html:187` |

## Goals / Non-Goals

Goals: one reachable system-default control on the live path; no second system control;
honest display of pinned agents; a written sub-agent rule.

Non-Goals: publishing the change to open clients or other replicas
(`uiux-realtime-topics-and-replica-refresh`); removing the `llm.default_provider` field and
the `models[0]` banner (`uiux-settings-truthful-status-and-removals`); the shared picker
component (`uiux-schema-controls-and-pickers`); team adoption of model changes
(`uiux-team-model-adoption-at-boundary`); any force or lock-to-system option.

## Decisions

### D1. Two writes from the console, model first

The route accepts only a provider id, so the pair needs the provider update (model) and
the route (provider). Order: `PUT` model, then `POST` default. A failure after the first
write leaves the live default unchanged; for a non-default provider the model write is
invisible to routing, and for the current default provider the model write is the change
itself. The reverse order would leave the new provider live with the old model.

The `PUT` replaces the whole configuration, so the action starts from a freshly fetched
provider config and treats a 409 as "provider changed, reload" rather than retrying with
stale data.

The `llm.default_provider` row is written only by this route (via `set_default_provider_id`,
`settings/manager.rs:1247-1249`) and by the boot seed when absent (`manager.rs:1018-1020`).
The settings API rejects writes to that key; that rejection is owned by
`uiux-settings-truthful-status-and-removals`. Replicas refresh the registry default from the
row; that is owned by `uiux-realtime-topics-and-replica-refresh`. This change relies on both
and issues no write to the key from the console.

Alternative considered: extend the route to accept `{ model }` and apply both under the
administration guard (`providers.rs:276`) for atomicity. Rejected for this phase because it
changes a pinned wire contract and the replica-refresh change already consumes the current
route's persistence; recorded as a follow-up if partial failure proves common.

### D2. Retire `run_policy.global.model` at the resolver

The single enforcement point is `resolve_run_policy`: skip the `model` branch for the
Global scope. This covers service and embedded paths because both reach it through
`resolve_effective_run_policy_core` (`policy.rs:683-689`, `runtime/manager.rs:3157`,
`api/discovery.rs:867`); the legacy fallback without a settings manager never reads Global. Stored values stay inert; whether Plan deletes them in a migration is
left open (the migration must not touch other keys, `gotchas.md:778-786`). Operator default for open
decision 3 (`spec-open-decisions.md`): ignore a non-null Global `model` on write and leave
stored values inert. Scenarios therefore never create the stored value through the
settings API; SCN-SYS-004 seeds it directly in the scratch store and asserts it reads back
before the turn. Rejecting on write would make the retirement visible to API clients but
break any caller that still sends it.

Alternative considered: keep it as an internal-only default. Rejected as the default
because it leaves two system levels that can disagree and a displayed default that is not
the one in force.

### D3. Embedded hosts

`embedded-run-policy-and-admin-surface` made `run_policy.global.model` effective on
embedded hosts on purpose. Retiring it changes those hosts to the registry default, which
on embedded builds is the on-device provider (`runtime/manager.rs:11338-11344` comment).
Whether any first-party embedded host writes Global `model` today is UNVERIFIED. If one
does, the operator must choose between this retirement and keeping the field for
embedded use only; this change assumes retirement.

### D4. Pinned-agent rule

Use the runtime's rule, not the display rule: explicit when the artifact default has both
provider and model, or the `uar.run_policy` extension carries a model. Whether the agents
read model exposes `extensions` to the console is UNVERIFIED; if it does not, add a
derived read-only `explicit_model` projection to the agents read and have the Agents list
and the system default panel use it. The same predicate replaces
`agentModelStatus` (`agents-page.tsx:206-225`).

### D5. Placement

The panel sits at the top of the Providers destination, matching the visual authority's
"Default chat" role card. The command palette gets an action entry in addition to its
navigation destinations. The Models-page star and the Providers "make default" action call
the one action, so no second write path survives.

### D6. Sub-agent rule

Source already behaves this way (`policy_intersection.rs:582-590`). The change writes the
rule down, adds a visible failure requirement (no substitution), and marks the rule pending
operator confirmation. The rule belongs in `multi-agent-orchestration` conceptually; it is
carried in `session-configuration` because that is the delta this change owns, and
`uiux-team-model-adoption-at-boundary` holds the delta on `multi-agent-orchestration`.

## Dependencies and interfaces

| Dependency | Direction | What this change needs from it |
|---|---|---|
| `uiux-realtime-topics-and-replica-refresh` | consumed | Open clients and other replicas learn of the default change; replicas refresh the registry default from the `llm.default_provider` row the route wrote. Also supplies the two-replica remote-store fixture SCN-SYS-009 uses. Until it lands, the console that makes the change refetches provider metadata itself after the writes; other clients and replicas may show or use the old default. |
| `uiux-settings-truthful-status-and-removals` | consumed | `llm.default_provider` leaves the settings UI and the settings API rejects writes to it (status and body owned there; the error points to `POST /api/uar/providers/{id}/default`); Providers banner shows `default_model`. SCN-SYS-007 uses that rejection as its negative control. |
| `uiux-schema-controls-and-pickers` | consumed | The shared combobox picker (candidate cand-002). If it has not landed, reuse `features/models/ui/model-selector.tsx`. |
| `uiux-model-identity-graph-ownership` | consumed | Provider-qualified Model identity and `ProviderMeta` ownership in the graph. |

Interface effects: no new HTTP route, no schema change. One added rejection: the default route
returns 409 `Provider '{id}' is disabled` for a disabled provider (task 1.7). The console's request set for a
system default change is `PUT /api/uar/providers/{id}` (when the model differs) then
`POST /api/uar/providers/{id}/default`. Resolver behavior change: Global `model` ignored.

## Risks

- Embedded hosts that rely on Global `model` (D3). Mitigation: operator decision in Plan.
- The YAML fallback `llm_config.model` in `resolve_default_model`
  (`runtime/manager.rs:11323-11331`) can still route a turn when the registry has no
  usable pair, so the console may say "no usable default" while the runtime resolves.
  Out of scope here; open decision whether to retire that fallback.
- A change made through one replica is not applied on others until the realtime change
  lands. Most SYS scenarios run one replica; SCN-SYS-009 runs two on one remote store, and the replica refresh of the registry default is accepted in the realtime change.
- A stale-revision 409 on the model write when two operators edit the same provider.

## Open decisions

1. Operator confirmation of the sub-agent rule (marked pending throughout).
2. Embedded hosts and Global `model` (D3).
3. Reject or ignore a non-null Global `model` on write; delete or leave stored values (D2). Operator default: ignore, leave inert.
4. Whether to retire the `llm_config.model` fallback.

## UNVERIFIED
- Whether `ProviderConfig` reads and PUT bodies carry the fields the picker needs (`enabled`, `models`) in the shape the frozen fixture assumes.
- How the SCN-SYS-008 control withholds a provider credential from the delegating thread.
- That the stub logs a request even when no fixture matches (`stub_llm.rs:195-200` reads as logging all bodies).

- Whether the agents read exposes `extensions["uar.run_policy"]` (D4).
- Whether a sub-agent run exposes its effective model to an observer (SYS-008 oracle).
- Runtime behavior of any scenario; nothing was executed.

## Acceptance

All rows are bound to the phase's final integration gate. Real collaborators only. Identity is provided by the existing external proxy; no scenario creates, sends or renews a credential or JWT. The model vendor is replaced by `stub-llm` (`src/bin/stub-llm.rs`; invocation: `target/debug/stub-llm <fixtures.json>` with `STUB_LLM_PORT`; request log `GET /_stub/requests`, reset `POST /_stub/requests`, `tests/integration/live/stub_llm.rs:186-203`). No scenario calls a paid model route.

### Behavior: Operator changes the system default and the next turn follows it (SCN-SYS-001)

| Field | Declaration |
|---|---|
| Canonical identity | uar-uiux-architecture-2026-11 / uiux-system-model-live-default / tasks 1.1-1.6, 2.1-2.2, 8.1 |
| Source identity | Repository revision at the implementation-complete inventory; the service and frontend bundle built from that tree |
| Production entry point | Console: Providers destination, System default model picker, select `bdd-provider-b/claude-5-haiku`, confirm; then message `S1 r1 after change` in a fresh conversation with `Default Assistant`. The provider's default_model is `claude-5-sonnet`, so the model must be saved before the live route is called |
| Real collaborators | UAR service process, scratch embedded store, served frontend in a real browser, provider routes, run manager; per-provider stub-llm instances (`target/debug/stub-llm`; request log `GET /_stub/requests` on each) |
| Boundary exercised | Browser to `PUT /api/uar/providers/bdd-provider-b` then `POST /api/uar/providers/bdd-provider-b/default`, in that order; service to the `bdd-provider-b` stub request carrying the model |
| Observable result | The stub of `bdd-provider-b` logs exactly one request for `S1 r1 after change` with model ending `claude-5-haiku` and the reply `bdd-reply-claude-5-haiku` is visible; no request for it carries `claude-5-sonnet`; while the harness holds `POST .../default` (page.route) no confirmation is visible, the system-default control does not show the new pair and a registry GET still returns `openai`; after release the confirmation appears with the requested value and the activation `live` (`confirmationComparator` in `fixtures/s1-oracle.json` SCN-SYS-001) |
| Negative control | Direct `POST /api/uar/providers/bdd-missing/default`: expected 404 body `Provider 'bdd-missing' not found`, default unchanged. Held-route control `CTL-SYS-001-HELD-500`: the held POST is fulfilled with 500 (it never reaches the server); no confirmation, a visible error saying the default was not changed, the control still showing `openai/gpt-5.4-mini`, no further write from the browser within 3 seconds, and a registry GET still returning `openai`. The GET and the no-further-write check guard only against a console that falls back to another write path that reaches the server (a second POST or a PUT of `llm.default_provider`); they cannot detect the 500 itself. Third control: the per-provider `Set as default` button on `bdd-provider-b` serves `claude-5-sonnet`, never `claude-5-haiku`, so the oracle rejects the legacy path. The button stays (it sets a provider default); no S1 oracle accepts it as S1-R1 |
| Isolation | Scratch data root; ephemeral service and per-provider stub ports; the harness removes all on exit |
| Prerequisites | Built `stub-llm` and `universal-agent-runtime`; `pnpm test:bdd` (playwright-bdd) with the S1 step definitions; frozen values in `fixtures/s1-oracle.json`; implementation-complete inventory recorded |
| Final local gate | `pnpm test:bdd` over `tests/bdd/features/live-model-choice.feature` and `live-model-choice-controls.feature`, each scenario with `--repeat-each=3 --retries=0` (every sample must pass), after the inventory; exact argv bound by the lead |
| Evidence | Pending |
| Limitations | One replica; vendor behavior not established (stub); the BDD run uses the deployment's auth-off test configuration and sends no credentials |

### Behavior: The system default survives restart (SCN-SYS-002)

| Field | Declaration |
|---|---|
| Canonical identity | uar-uiux-architecture-2026-11 / uiux-system-model-live-default / tasks 1.1-1.3, 8.2 |
| Source identity | Repository revision at the implementation-complete inventory; the service and frontend bundle built from that tree |
| Production entry point | Change the default in the console, stop and start the service on the same scratch root, read `GET /api/uar/providers`, send `uar-sys-ping-001` to `sys-agent-a` |
| Real collaborators | Service process, scratch store, provider routes, frontend, stub-llm |
| Boundary exercised | Process restart; `default_id` and provider `default_model` in the provider read; boot hydration |
| Observable result | After restart `default_id` = `sysp2`, `sysp2` default_model = `sys-m3`, and the stub log shows `sys-m3` for the next turn |
| Negative control | Conflicting scratch YAML (lists `sysp1` first with default `sys-m1`, `llm.model` = `sysp1/sys-m1`): expected rejection of the YAML default; the registry default after restart stays `sysp2`/`sys-m3` |
| Isolation | Scratch data root; ephemeral service and stub ports; scratch provider YAML generated from `fixtures/sys-oracle.json`; the harness removes all on exit; the scratch root survives the restart only |
| Prerequisites | Built `stub-llm` and `universal-agent-runtime` (`cargo build --bin stub-llm --bin universal-agent-runtime`, per `tests/bdd/playwright.config.ts:68`, build command added by the lead); real browser driver; implementation-complete inventory recorded |
| Final local gate | Phase production integration gate; exact argv bound in Plan; runs only after the inventory |
| Evidence | Pending |
| Limitations | Embedded store only; remote-store restart behavior is accepted in the config and realtime changes |

### Behavior: Agents with an explicit model keep it, and the console says so (SCN-SYS-003)

| Field | Declaration |
|---|---|
| Canonical identity | uar-uiux-architecture-2026-11 / uiux-system-model-live-default / tasks 4.1-4.3, 8.3 |
| Source identity | Repository revision at the implementation-complete inventory; the service and frontend bundle built from that tree |
| Production entry point | Console picker confirm to `bdd-provider-b/claude-5-haiku` with agents `Default Assistant`, `BDD Agent Dawn` (user-created, no model), `BDD Agent Aurora` (artifact model `openai/gpt-5.4-mini`) and `BDD Agent Cirrus` (extension-only model `bdd-provider-c/qwen-5-turbo`); then one message per agent |
| Real collaborators | Service, agents read route, run manager, frontend, per-provider stub-llm instances (`target/debug/stub-llm`; request log `GET /_stub/requests` on each) |
| Boundary exercised | Agents read to console list; per-agent turn to the serving provider's stub request |
| Observable result | Replies: Dawn and Default Assistant from `claude-5-haiku`, Aurora from `gpt-5.4-mini`, Cirrus from `qwen-5-turbo`. The console pinned list equals exactly {Aurora, Cirrus}, and the follow count equals the number of agents returned by the agents read minus 2 |
| Negative control | `Default Assistant` and the user-created `BDD Agent Dawn` must not appear in the pinned list; a predicate that pins every user-created agent fails on Dawn |
| Isolation | Scratch data root; ephemeral service and per-provider stub ports; the harness removes all on exit |
| Prerequisites | Built `stub-llm` and `universal-agent-runtime`; `pnpm test:bdd` (playwright-bdd) with the S1 step definitions; frozen values in `fixtures/s1-oracle.json`; implementation-complete inventory recorded |
| Final local gate | `pnpm test:bdd` over `tests/bdd/features/live-model-choice.feature` and `live-model-choice-controls.feature`, each scenario with `--repeat-each=3 --retries=0` (every sample must pass), after the inventory; exact argv bound by the lead |
| Evidence | Pending |
| Limitations | Team-member and conversation overrides are not listed |

### Behavior: run_policy.global.model no longer selects a model (SCN-SYS-004)

| Field | Declaration |
|---|---|
| Canonical identity | uar-uiux-architecture-2026-11 / uiux-system-model-live-default / tasks 3.1-3.4, 8.4 |
| Source identity | Repository revision at the implementation-complete inventory; the service and frontend bundle built from that tree |
| Production entry point | Scratch store seeded directly with `run_policy.global.model` = `sysp2`/`sys-m4` before the service starts; read `GET /api/uar/settings/presentation-policy`; turn to `sys-agent-a`; read the effective run policy |
| Real collaborators | Service, settings manager, run manager, effective run policy artifact, stub-llm; the store runner defined in `uiux-config-boot-safety-and-live-reload` for the out-of-band seed |
| Boundary exercised | Stored row to resolver to stub request |
| Observable result | Precondition: the presentation-policy read returns `policy.model` = the stored route. Then the stub log shows `sys-m1` (registry default), never `sys-m4`; provenance of the effective model is not Global; the stored value still reads back unchanged |
| Negative control | Conversation model `sysp1`/`sys-m2` set on the same conversation: expected turn model `sys-m2` |
| Isolation | Scratch data root; ephemeral service and stub ports; scratch provider YAML generated from `fixtures/sys-oracle.json`; the harness removes all on exit |
| Prerequisites | Built `stub-llm` and `universal-agent-runtime` (`cargo build --bin stub-llm --bin universal-agent-runtime`, per `tests/bdd/playwright.config.ts:68`, build command added by the lead); real browser driver; implementation-complete inventory recorded; the store runner from the config change |
| Final local gate | Phase production integration gate; exact argv bound in Plan; runs only after the inventory |
| Evidence | Pending |
| Limitations | Embedded SDK path is covered only if the gate includes an embedded host; otherwise the service path stands in and the evidence says so |

### Behavior: The picker is reachable from primary navigation (SCN-SYS-005)

| Field | Declaration |
|---|---|
| Canonical identity | uar-uiux-architecture-2026-11 / uiux-system-model-live-default / tasks 2.1-2.3, 8.5 |
| Source identity | Repository revision at the implementation-complete inventory; the service and frontend bundle built from that tree |
| Production entry point | Console landing page; primary navigation and command palette (action label `Change system default model`), at 320 and 1440 pixels; one keyboard-only run |
| Real collaborators | Frontend bundle served by the service, real browser |
| Boundary exercised | Browser UI |
| Observable result | The picker opens in at most 2 interactions without typing a URL (frozen limit in the fixture) |
| Negative control | `sysp2` made default then disabled by `PUT /api/uar/providers/sysp2` with enabled=false: the picker shows `No usable system default` and does not show `sysp2`/`sys-m3` as live |
| Isolation | Scratch data root; ephemeral service and stub ports; scratch provider YAML generated from `fixtures/sys-oracle.json`; the harness removes all on exit |
| Prerequisites | Built `stub-llm` and `universal-agent-runtime` (`cargo build --bin stub-llm --bin universal-agent-runtime`, per `tests/bdd/playwright.config.ts:68`, build command added by the lead); real browser driver; implementation-complete inventory recorded |
| Final local gate | Phase production integration gate; exact argv bound in Plan; runs only after the inventory |
| Evidence | Pending |
| Limitations | No screen-reader evidence |

### Behavior: Failed or rejected changes alter nothing and say so (SCN-SYS-006)

| Field | Declaration |
|---|---|
| Canonical identity | uar-uiux-architecture-2026-11 / uiux-system-model-live-default / tasks 1.1-1.7, 8.6 |
| Source identity | Repository revision at the implementation-complete inventory; the service and frontend bundle built from that tree |
| Production entry point | Console confirm with the scratch remote Surreal store stopped. Part A: `sysp2` default_model already `sys-m3` (seeded in the scratch store), selection `sysp2`/`sys-m3`. Part B: `sysp2` default_model `sys-m4`, same selection |
| Real collaborators | Two service replicas on one scratch remote Surreal store, provider routes, `/api/live` on both replicas, frontend; the store process the harness stops (no product fault hook) |
| Boundary exercised | Browser to `POST /api/uar/providers/sysp2/default` (part A) and to `PUT /api/uar/providers/sysp2` (part B); persistence failure; console error state |
| Observable result | Part A: the console issues only the default route call; it returns 500 with a body starting `Failed to persist default provider`; `default_id` is `sysp1` on both replicas; after restore the `llm.default_provider` row is `sysp1`; no `providers` or `settings` invalidation frame arrives on either replica for 10 seconds; the console shows the error and not `sysp2`/`sys-m3` as live. Part B: the provider save returns 500 with a body starting `provider_persist_failed`; the browser makes no default route call; the console shows the save error and says the default was not changed; `default_id` is `sysp1` on both replicas |
| Negative control | Direct `POST /api/uar/providers/sysp3/default` with `sysp3` disabled and the store healthy: expected 409 `Provider 'sysp3' is disabled`, registry and store unchanged. A route without the disabled check (the pre-phase baseline accepts it with 200 and swaps the default) fails this control. The baseline cannot control parts A and B: its route already persists before swapping and its console rolls back the optimistic default, so those parts are failed instead by a console that shows the pair as live after the failure, calls the default route after a failed save, or a route that swaps before persisting |
| Isolation | Scratch data root, ephemeral ports, two replicas and a scratch remote store as separate processes the harness starts and stops; removed on exit |
| Prerequisites | Built service and frontend; real browser driver; the two-replica remote-store fixture from `uiux-realtime-topics-and-replica-refresh`; BLOCKED, never skipped, when unavailable |
| Final local gate | Phase production integration gate; exact argv bound in Plan; runs only after the inventory |
| Evidence | Pending |
| Limitations | While the store is stopped the change feed itself is down, so the no-invalidation assertion is evaluated after restore over a fixed 10-second window and is weak during the outage; the partial outcome where the model save succeeds and the default route then fails needs a failure between the two calls and is not exercised |

### Behavior: The console issues no competing system-default write (SCN-SYS-007)

| Field | Declaration |
|---|---|
| Canonical identity | uar-uiux-architecture-2026-11 / uiux-system-model-live-default / tasks 1.1-1.6, 8.7 |
| Source identity | Repository revision at the implementation-complete inventory; the service and frontend bundle built from that tree |
| Production entry point | Complete the S1-R1 change with the browser's request log on; then a direct settings write by the harness |
| Real collaborators | Frontend, service, settings API, request log, provider read-backs |
| Boundary exercised | Browser to HTTP; settings API write rejection |
| Observable result | The browser's `/api/uar/**` write requests equal exactly, in order, {`PUT /api/uar/providers/bdd-provider-b`, `POST /api/uar/providers/bdd-provider-b/default`}; none targets `llm.default_provider`, `llm.model` or `run_policy.global` |
| Negative control | Harness `PUT /api/uar/settings/llm.default_provider` naming `bdd-provider-b`: expected 4xx whose body contains `/api/uar/providers/` and `default` (code and text owned by `uiux-settings-truthful-status-and-removals`); registry `default_id` and the stored `llm.default_provider` row read back unchanged |
| Isolation | Scratch data root; ephemeral service and per-provider stub ports; the harness removes all on exit |
| Prerequisites | Built `stub-llm` and `universal-agent-runtime`; `pnpm test:bdd` (playwright-bdd) with the S1 step definitions; frozen values in `fixtures/s1-oracle.json`; implementation-complete inventory recorded |
| Final local gate | `pnpm test:bdd` over `tests/bdd/features/live-model-choice.feature` and `live-model-choice-controls.feature`, each scenario with `--repeat-each=3 --retries=0` (every sample must pass), after the inventory; exact argv bound by the lead |
| Evidence | Pending |
| Limitations | The exact rejection code is the settings change's; this scenario checks only that a rejection with a pointer to the route occurs and nothing changes |

### Behavior: A sub-agent's explicit model wins (SCN-SYS-008, pending operator confirmation)

| Field | Declaration |
|---|---|
| Canonical identity | uar-uiux-architecture-2026-11 / uiux-system-model-live-default / tasks 5.1-5.3, 8.8 |
| Source identity | Repository revision at the implementation-complete inventory; the service and frontend bundle built from that tree |
| Production entry point | Conversation with saved model `sysp1`/`sys-m2`; `sys-agent-root` delegates to `sys-agent-child-own` (model `sysp2`/`sys-m4`) and `sys-agent-child-none`, through the shipped delegation path (exact path UNVERIFIED) |
| Real collaborators | Service, run manager, thread policy intersection, stub-llm |
| Boundary exercised | Delegation to child run to stub request |
| Observable result | Stub log by run: root `sys-m2`, own-model child `sys-m4`, no-model child `sys-m2` |
| Negative control | `sys-agent-child-nocred` (model on `sysp4`, credential not granted): delegation fails visibly; the stub log holds no request for `sys-m6` and none from that delegation for any other model |
| Isolation | Scratch data root; ephemeral service and stub ports; scratch provider YAML generated from `fixtures/sys-oracle.json`; the harness removes all on exit |
| Prerequisites | Operator confirmation of the rule; Built `stub-llm` and `universal-agent-runtime` (`cargo build --bin stub-llm --bin universal-agent-runtime`, per `tests/bdd/playwright.config.ts:68`, build command added by the lead); real browser driver; implementation-complete inventory recorded |
| Final local gate | Phase production integration gate; exact argv bound in Plan; runs only after the inventory |
| Evidence | Pending |
| Limitations | Not mandatory until the rule is confirmed; observability of the child model and the way the credential is withheld are UNVERIFIED |

### Behavior: An agent model edit applies to the next turn, on another replica too (SCN-SYS-009)

| Field | Declaration |
|---|---|
| Canonical identity | uar-uiux-architecture-2026-11 / uiux-system-model-live-default / tasks 4.4, 8.9 |
| Source identity | Repository revision at the implementation-complete inventory; the service and frontend bundle built from that tree |
| Production entry point | On replica 1 edit `sys-agent-a` to model `sysp2`/`sys-m4` through the agent editor; send `uar-sys-ping-001` in an open conversation served by replica 2 |
| Real collaborators | Two service replicas on one scratch remote Surreal store, agents routes, run manager, frontend, stub-llm |
| Boundary exercised | Agent write on one replica to turn resolution on another to stub request |
| Observable result | Stub log: before the edit `sys-m1`; after the edit the next turn on replica 2 matches `sys-m4` |
| Negative control | Edit the agent to `sysp2`/`sys-no-such-model`: the next turn reports an error, the stub log gains no request, and `sys-m1` (the system default) is not used |
| Isolation | Scratch data root; ephemeral service and stub ports; scratch provider YAML generated from `fixtures/sys-oracle.json`; the harness removes all on exit; two replicas and the remote store are separate scratch processes |
| Prerequisites | Built `stub-llm` and `universal-agent-runtime` (`cargo build --bin stub-llm --bin universal-agent-runtime`, per `tests/bdd/playwright.config.ts:68`, build command added by the lead); real browser driver; implementation-complete inventory recorded; the two-replica remote-store fixture defined in `uiux-realtime-topics-and-replica-refresh` |
| Final local gate | Phase production integration gate; exact argv bound in Plan; runs only after the inventory |
| Evidence | Pending |
| Limitations | Relies on agents being re-read from the store each turn (assessment G4, `server.rs:7609-7737`; not re-read here); the expected error for an unroutable agent model is UNVERIFIED |

### Behavior: The picker stays responsive over the full catalog (SCN-SYS-010)

| Field | Declaration |
|---|---|
| Canonical identity | uar-uiux-architecture-2026-11 / uiux-system-model-live-default / tasks 1.9, 8.10 |
| Source identity | Repository revision at the implementation-complete inventory; the frontend bundle built from that tree, plus the baseline bundle from revision `f2c9e36efe088e06a17c5d880ada877e25f2ad52` |
| Production entry point | Console: system default picker opened with the bundled catalog loaded (8,170 pairs), then the query `claude-opus` typed |
| Real collaborators | Service with the bundled catalog (`/api/models`), served frontend in a real browser with a long-task observer; configured providers from the scratch YAML |
| Boundary exercised | Browser rendering and input; `/api/models` to graph to picker |
| Observable result | Visible count text `347 models`; at most 200 option elements in the DOM unfiltered and filtered; no long task above 200 ms during open and typing (frozen in `fixtures/sys-oracle.json` `catalogScale`) |
| Negative control | Baseline bundle's Models page (`/admin/models`, full-catalog list) under the same query and measurement: expected more than 200 rendered rows or a long task above 200 ms. If the baseline passes both, BLOCKED |
| Isolation | Scratch root, ephemeral ports; baseline worktree `~/.claude/worktrees/sys-baseline` created with `scripts/worktree-new.sh sys-baseline --base f2c9e36efe088e06a17c5d880ada877e25f2ad52` and removed with `scripts/worktree-rm.sh` |
| Prerequisites | Built service and both frontend bundles; real browser driver; the catalog snapshot digest equals the fixture (else BLOCKED); how the runner selects the baseline bundle is UNVERIFIED and bound by the lead |
| Final local gate | Phase production integration gate; exact argv bound in Plan |
| Evidence | Pending |
| Limitations | The 200 ms budget is proposed and not yet accepted by the operator; a long-task observer does not measure input latency below the browser's 50 ms long-task floor; the control is a different surface than the candidate, so it proves the measurement can reject, not that the baseline had this picker |

## Frozen oracle fixtures

Expected values are fixed now, in files the boundary runner reads, and are not read from
implementation code, route tables, classifiers or the system under test.

| File | Content |
|---|---|
| `openspec/changes/uiux-system-model-live-default/fixtures/sys-oracle.json` (sha256 `b6e6ad344849cdc080b0f6ea047023a60b8ae757a5c31d32693b3abd2a6f0737`; SCN-SYS-001, 003, 007 moved to `s1-oracle.json`) | Providers `sysp1`-`sysp4` with models and initial defaults, the initial and selected system default pair, the exact allowed console write set, agents and the expected pinned set, expected stub model per scenario step, negative-control inputs and expected rejections, picker interaction limit and strings, sample counts, and the catalog-scale block (snapshot digest, frozen query `claude-opus` with 347 expected matches, thresholds, baseline control) |
| `openspec/changes/uiux-system-model-live-default/fixtures/s1-oracle.json` | S1 vocabulary (`bdd-provider-*`, `BDD Agent *`): models A, B, C, the unservable choices, agents, origin labels, stub entries required, and the frozen values and controls of SCN-SYS-001, 003, 007, 011-017, stub instances, new stub kinds and the receipt time comparator, the confirmation comparator and the mutant definitions. sha256 `39a2d8c239362e2d36a1d62ded8401ec2c8e172ee4f6c599c2719c42130e6f9b` |
| `openspec/changes/uiux-system-model-live-default/fixtures/baseline-verdicts.json` | Expected pre-phase verdicts for the mandatory scenarios including SCN-SYS-011..017 (round-1 S1 revision). sha256 `4346f0a55c4b556898412e14aaffd9d028c41469d3c1865cb30b8272afe1f635` |
| `openspec/changes/uiux-system-model-live-default/fixtures/stub-llm-sys.json` (sha256 `62f0e98a3ec5c28d070ed065ca08bcbc402ba49f023edfd94f7374f2f9ad537e`) | stub-llm fixture entries for models `sys-m1`..`sys-m4` (bare and `provider/`-qualified, with and without tools) for the user message `uar-sys-ping-001` |

Bare model ids are unique across the four providers, so a stub-log request matches a model
when `body.model` equals the bare id or `<providerId>/<bare id>`. Values marked UNVERIFIED in
this document are not fixture values and do not become oracle truth until verified.

## Scenario matrix rows

Common to all rows (SCN-SYS-001 to SCN-SYS-009; SCN-SYS-010 states its own samples): real collaborators are the service process (two replicas on a scratch remote store for SCN-SYS-009 and SCN-SYS-006), scratch store, served frontend in a real browser, the provider and run routes and `stub-llm`; fixtures are a scratch data root, ephemeral ports and the frozen files in `fixtures/` (providers `sysp1`-`sysp4`, agents `sys-agent-*`); restoration is stopping the services, the stub and any scratch store and deleting the scratch root; prerequisites are the built `stub-llm` and service binaries (build commands added by the lead). Every row is `nondeterministic: true` (network, browser rendering, model-output timing, restarts) with `stochastic`: samples = 3, grader = the row's oracle comparator holds and its negative control rejects in that sample, stoppingRule = fixed N=3, all must pass. A mandatory row whose negative control did not run is BLOCKED, never PASS.

**SCN-SYS-001. Operator changes the system default and the next turn follows it**
- Actor: console operator. Useful outcome: the whole system moves to the chosen model for every agent without a model, no restart.
- Source requirement: `openspec:provider-model-settings-certification#The system default model is the registry default provider plus its default model`
- Reason now: decisions 2 and 9; today the only reachable path is a boot-only field. Likelihood: UNKNOWN. Mandatory: true. S1 vocabulary (`fixtures/s1-oracle.json`).
- Production entry: console picker on the Providers destination selecting a model that differs from the provider's default_model, then `S1 r1 after change` in a fresh conversation with `Default Assistant`.
- Oracle: observation = the `bdd-provider-b` stub's single request for the message, the visible reply text, the confirmation's presence while the live route is held and after release, requested value and activation label, registry GET while held; truth source = `fixtures/s1-oracle.json` SCN-SYS-001 (served `claude-5-haiku`, never `claude-5-sonnet`; writes PUT then POST); comparator = exact request count and model-suffix match, ordered write list.
- Negative control: input = direct `POST /api/uar/providers/bdd-missing/default`, expected 404 and no state change; second input = control `CTL-SYS-001-HELD-500` (held route answered with 500; expected no confirmation, a visible error, registry GET `openai`); third input = the per-provider button, expected to serve `claude-5-sonnet` so the oracle rejects it as S1-R1.
- Nondeterministic: true. Stochastic: samples 3 (`--repeat-each=3 --retries=0`); grader per sample = oracle holds and control rejects; stoppingRule fixed N=3, all must pass.
- Evidence required: browser request log with ordering, stub request log of each provider, console screenshot, service log lines.
- Substituted boundaries: model vendor (stub-llm).
- Limitations: one replica; no push to other clients.
- Invalidation keys: `src/uar/api/providers.rs`, `src/llm/registry.rs`, `src/uar/runtime/manager.rs`, `frontend/src/features/providers/**`, `tests/bdd/features/live-model-choice*.feature`, `fixtures/s1-oracle.json`.
- Owner tasks: 1.1-1.6, 2.1-2.2, 7.9-7.11, 8.1.

**SCN-SYS-002. The system default survives restart**
- Actor: operator. Useful outcome: the chosen default is still in force after a restart.
- Source requirement: `openspec:provider-model-settings-certification#The system default model is the registry default provider plus its default model`
- Reason now: persistence is split across the `llm.default_provider` row and the provider row. Likelihood: UNKNOWN. Mandatory: true.
- Production entry: change the default, restart the service on the same scratch root, read providers, send a turn to `sys-agent-a`.
- Oracle: observation = `default_id`, provider `default_model` and stub model after restart; truth source = the pair selected before restart (fixture `selectedSystemDefault`); comparator = equality.
- Negative control: input = conflicting scratch YAML (`sysp1` first, `llm.model` = `sysp1/sys-m1`); expected rejection = the YAML default does not win, registry default stays `sysp2`/`sys-m3`.
- Nondeterministic: true. Stochastic: samples 3; grader per sample as above; stoppingRule fixed N=3, all must pass.
- Evidence required: pre and post restart provider reads, stub log, process start and stop receipts.
- Substituted boundaries: model vendor (stub-llm).
- Limitations: embedded store only.
- Invalidation keys: `src/uar/settings/provider_sync.rs`, `src/uar/settings/manager.rs`, `src/server.rs`, config seeding files, `fixtures/**`.
- Owner tasks: 1.1-1.3, 8.2.

**SCN-SYS-003. Agents with an explicit model keep it and the console says so**
- Actor: operator. Useful outcome: the operator sees that a change did not reach pinned agents and the runtime agrees.
- Source requirement: `openspec:session-configuration#An explicit model beats the system default; openspec:session-configuration#The console shows which agents keep an explicit model when the default changes`
- Reason now: decision 9 cost statement. Likelihood: UNKNOWN. Mandatory: true. S1 vocabulary (`fixtures/s1-oracle.json`).
- Production entry: console picker confirm to `bdd-provider-b/claude-5-haiku`, then one message to each of `Default Assistant`, `BDD Agent Dawn`, `BDD Agent Aurora`, `BDD Agent Cirrus`.
- Oracle: observation = serving stub request per agent and the console pinned list and follow count; truth source = `fixtures/s1-oracle.json` SCN-SYS-003 and `agents`; comparator = exact set equality {Aurora, Cirrus}, follow count = agents read minus 2, per-agent model equality.
- Negative control: input = `Default Assistant` and the user-created `BDD Agent Dawn` (no model); expected rejection = neither listed as pinned, both follow `claude-5-haiku`.
- Nondeterministic: true. Stochastic: samples 3 (`--repeat-each=3 --retries=0`); grader per sample = oracle holds and control rejects; stoppingRule fixed N=3, all must pass.
- Evidence required: stub logs per agent, console screenshot, agents read response.
- Substituted boundaries: model vendor (stub-llm).
- Limitations: team-member and conversation overrides not listed.
- Invalidation keys: `src/uar/domain/policy.rs`, `frontend/src/features/agents/**`, agents read route files, `fixtures/s1-oracle.json`.
- Owner tasks: 4.1-4.3, 8.3.

**SCN-SYS-004. run_policy.global.model no longer selects a model**
- Actor: operator or SDK client. Useful outcome: a stale stored global model cannot override the registry default.
- Source requirement: `openspec:provider-model-settings-certification#run_policy.global.model is retired from model selection`
- Reason now: decision 2. Likelihood: UNKNOWN. Mandatory: true.
- Production entry: scratch store seeded out of band with `run_policy.global.model` = `sysp2`/`sys-m4`; read `GET /api/uar/settings/presentation-policy`; turn on `sys-agent-a`; read the effective run policy.
- Oracle: observation = presentation-policy read (precondition: `policy.model` equals the stored route), stub model, effective-policy provenance, stored value after; truth source = fixture SCN-SYS-004 (registry default `sys-m1`, forbidden `sys-m4`); comparator = equality plus provenance is not Global.
- Negative control: input = conversation model `sysp1`/`sys-m2` on the same conversation; expected rejection of the global value = turn model `sys-m2`.
- Nondeterministic: true. Stochastic: samples 3; grader per sample as above; stoppingRule fixed N=3, all must pass.
- Evidence required: presentation-policy read before and after, stub log, effective run policy artifact.
- Substituted boundaries: model vendor (stub-llm); the stored value is written out of band (the product cannot create it under the ignore-on-write default).
- Limitations: embedded SDK path only if the gate includes an embedded host.
- Invalidation keys: `src/uar/domain/policy.rs`, `src/uar/settings/manager.rs`, `src/embedded.rs`, `fixtures/**`.
- Owner tasks: 3.1-3.4, 8.4.

**SCN-SYS-005. The picker is reachable from primary navigation**
- Actor: operator. Useful outcome: the control can be found and used without a URL.
- Source requirement: `openspec:provider-model-settings-certification#The system default picker is reachable from primary navigation`
- Reason now: the Models-page control is unreachable. Likelihood: UNKNOWN. Mandatory: true.
- Production entry: console landing page; navigation and palette (`Change system default model`) at 320 and 1440 pixels; one keyboard-only run.
- Oracle: observation = interaction count and focus path to the open picker; truth source = fixture `maxInteractions` = 2; comparator = count <= 2.
- Negative control: input = `sysp2` default then disabled via `PUT /api/uar/providers/sysp2` (enabled=false); expected = picker shows `No usable system default` and not `sysp2`/`sys-m3` as live.
- Nondeterministic: true. Stochastic: samples 3; grader per sample as above; stoppingRule fixed N=3, all must pass.
- Evidence required: browser trace, screenshots at both widths.
- Substituted boundaries: none.
- Limitations: no screen-reader evidence.
- Invalidation keys: `frontend/src/app/shell/**`, `frontend/src/features/providers/ui/**`, `fixtures/**`.
- Owner tasks: 2.1-2.3, 8.5.

**SCN-SYS-006. Failed or rejected changes alter nothing and say so**
- Actor: operator. Useful outcome: a failure never shows a default that is not in force, on any replica.
- Source requirement: `openspec:provider-model-settings-certification#Configuration routes real work` (scenario: Failed persistence changes no live state); `openspec:provider-model-settings-certification#The system default model is the registry default provider plus its default model`; `openspec:provider-model-settings-certification#The default route rejects a disabled provider`
- Reason now: the spec forbids live change on failed persistence and the console must reflect it; a correct console makes one write when the model already matches and none after a failed save. Likelihood: UNKNOWN. Mandatory: true.
- Production entry: console confirm of `sysp2`/`sys-m3` with the scratch remote store stopped, part A (model already `sys-m3`, only the default route is called) and part B (model `sys-m4`, the provider save fails).
- Oracle: observation = the browser's write requests, HTTP status and body prefix, `default_id` on both replicas, the `llm.default_provider` row after restore, `/api/live` frames on both replicas for 10 seconds after restore, console state; truth source = fixture SCN-SYS-006 `partA_routePersistFails` and `partB_saveFails`; comparator = exact request set, status 500, prefixes `Failed to persist default provider` and `provider_persist_failed`, pair equals `sysp1`/`sys-m1` on both replicas, no providers or settings frame.
- Negative control: input = direct `POST /api/uar/providers/sysp3/default` with `sysp3` disabled; expected rejection = 409 body `Provider 'sysp3' is disabled`, registry and store unchanged. The baseline cannot control parts A and B (its route persists before swapping, its console rolls back); see the Acceptance table for the wrong implementations those parts fail.
- Nondeterministic: true. Stochastic: samples 3; grader per sample = both parts and the control hold; stoppingRule fixed N=3, all must pass.
- Evidence required: browser request log, responses, provider reads from both replicas before, during and after, frame logs, store stop and start receipts, screenshots.
- Substituted boundaries: none; the scratch remote store is real.
- Limitations: the no-invalidation check is weak during the outage and is evaluated after restore; the model-saved-then-route-failed outcome is not exercised.
- Invalidation keys: `src/uar/api/providers.rs`, `src/llm/registry.rs`, `src/uar/settings/manager.rs`, `frontend/src/features/providers/**`, `frontend/src/lib/realtime/optimistic.ts`, `fixtures/**`.
- Owner tasks: 1.1-1.7, 8.6.

**SCN-SYS-007. The console issues no competing system-default write**
- Actor: operator. Useful outcome: one write path to the system default.
- Source requirement: `openspec:provider-model-settings-certification#The system default model is the registry default provider plus its default model`
- Reason now: analysis delta 5; the settings API rejection is owned by `uiux-settings-truthful-status-and-removals`. Likelihood: UNKNOWN. Mandatory: true. S1 vocabulary (`fixtures/s1-oracle.json`).
- Production entry: complete the S1-R1 change with the browser request log on, then a direct settings write by the harness.
- Oracle: observation = ordered write requests to `/api/uar/**` from the browser, registry and stored-row read-backs after the direct write; truth source = `fixtures/s1-oracle.json` SCN-SYS-007; comparator = ordered set equality and no forbidden key written.
- Negative control: input = harness `PUT /api/uar/settings/llm.default_provider` naming `bdd-provider-b`; expected rejection = 4xx whose body contains `/api/uar/providers/` and `default`; registry `default_id` and stored row unchanged.
- Nondeterministic: true. Stochastic: samples 3 (`--repeat-each=3 --retries=0`); grader per sample = oracle holds and control rejects; stoppingRule fixed N=3, all must pass.
- Evidence required: request log, rejection response, provider and settings reads after.
- Substituted boundaries: none.
- Limitations: exact rejection code owned by the settings change.
- Invalidation keys: `frontend/src/features/providers/**`, `frontend/src/features/settings/**`, `src/uar/api/settings.rs`, `fixtures/s1-oracle.json`.
- Owner tasks: 1.1-1.6, 8.7.

**SCN-SYS-008. A sub-agent's explicit model wins (pending operator confirmation)**
- Actor: operator running a delegating agent. Useful outcome: delegated work uses the model its definition names.
- Source requirement: `openspec:session-configuration#A sub-agent's explicit model wins over conversation and system defaults (pending operator confirmation)`
- Reason now: the rule is unspecified; analysis item 16. Likelihood: UNKNOWN. Mandatory: false until the operator confirms the rule.
- Production entry: conversation with saved model `sysp1`/`sys-m2`; `sys-agent-root` delegates to the fixture children through the shipped delegation path (exact path UNVERIFIED).
- Oracle: observation = stub model per request tagged by run; truth source = fixture SCN-SYS-008; comparator = root `sys-m2`, own-model child `sys-m4`, no-model child `sys-m2`.
- Negative control: input = `sys-agent-child-nocred` (model on `sysp4`, credential not granted); expected rejection = visible delegation failure and no stub request for `sys-m6` or any substitute.
- Nondeterministic: true. Stochastic: samples 3; grader per sample as above; stoppingRule fixed N=3, all must pass.
- Evidence required: stub log by run, delegation result record.
- Substituted boundaries: model vendor (stub-llm).
- Limitations: observability of the child's model and how the credential is withheld are UNVERIFIED.
- Invalidation keys: `src/uar/runtime/thread/policy_intersection.rs`, `src/uar/runtime/manager.rs`, `fixtures/**`.
- Owner tasks: 5.1-5.3, 8.8.

**SCN-SYS-009. An agent model edit applies to the next turn, on another replica too**
- Actor: operator. Useful outcome: editing an agent's model changes what its next turn uses, wherever the turn is served.
- Source requirement: `openspec:session-configuration#An agent model change applies to the next turn`
- Reason now: goal 4 and the release claim name this action; no earlier scenario covers it. Likelihood: UNKNOWN. Mandatory: true.
- Production entry: edit `sys-agent-a` on replica 1 in the agent editor; message in an open conversation served by replica 2.
- Oracle: observation = stub model before and after on replica 2; truth source = fixture SCN-SYS-009 (before `sys-m1`, after `sys-m4`); comparator = model-match rule.
- Negative control: input = edit to `sysp2`/`sys-no-such-model`; expected rejection = turn error, no stub request, no use of `sys-m1`.
- Nondeterministic: true. Stochastic: samples 3; grader per sample as above; stoppingRule fixed N=3, all must pass.
- Evidence required: stub log, agent read from both replicas, turn error text.
- Substituted boundaries: model vendor (stub-llm).
- Limitations: relies on agents re-read each turn (assessment G4); expected error text for an unroutable agent model UNVERIFIED.
- Invalidation keys: `src/server.rs`, `src/uar/api/discovery.rs`, `src/uar/domain/policy.rs`, `frontend/src/features/agents/**`, `fixtures/**`.
- Owner tasks: 4.4, 8.9.

**SCN-SYS-010. The picker stays responsive over the full catalog**
- Actor: operator. Useful outcome: the operator can find and choose a model among 8,170 catalog entries without the page freezing.
- Source requirement: `openspec:provider-model-settings-certification#The system default picker handles the full model catalog`
- Reason now: this is the only picker that lists the full catalog; the pickers change defers its scale check here. Likelihood: UNKNOWN. Mandatory: true.
- Production entry: open the system default picker with the bundled catalog and type `claude-opus`.
- Oracle: observation = visible count text, option-element count, long tasks during open and typing; truth source = `fixtures/sys-oracle.json` `catalogScale` (347 matches, at most 200 elements, 200 ms), derived from the catalog snapshot alone; comparator = equality for the count, upper bounds for elements and long task.
- Negative control: input = baseline bundle's Models page, same query and measurement; expected rejection = more than 200 rendered rows or a long task above 200 ms, else BLOCKED.
- Nondeterministic: true. Stochastic: candidate samples 5; control samples 3; grader per candidate sample = count 347 and elements at most 200 in both states and max long task at most 200 ms; per control sample = at least one bound exceeded; stoppingRule fixed N, all must pass.
- Evidence required: option dump, long-task list, count text, snapshot digest, baseline run report.
- Substituted boundaries: none.
- Limitations: budget is proposed; 50 ms long-task floor; control is a different surface.
- Invalidation keys: `catalog/provider_catalog.json`, `frontend/src/features/providers/**`, `frontend/src/components/ui/**`, `frontend/src/features/models/**`, `fixtures/**`.
- Owner tasks: 1.9, 8.10.

## S1 value rules

Slice S1 of the re-anchored phase (`re-anchor-2026-11-19.md`): live model choice governs the
next eligible response. Traces `docs/uar-settings-user-goals/specification.md` R-02, R-03,
R-06, R-07 and V-01 (local, proposed, not adopted) and memory record 02 (requested versus
effective, origin, activation boundary, clear-to-inherit, in-flight turns stay bound, never
report the requested candidate as served).

Identity: S1 uses the deployment's operator authority only. BDD runs use the deployment's
auth-off test configuration (`tests/bdd/playwright.config.ts`) and send no credentials; no
external identity proxy is involved in these runs. No My Defaults (U scope).

Frozen example values (synthetic; stub providers only; each provider has its own stub
instance so the request log names the serving provider):

| Name | Value | Wire model id |
|---|---|---|
| Model A, the initial system default and the YAML fallback | `openai/gpt-5.4-mini` | `gpt-5.4-mini` |
| Model B, the new system default | `bdd-provider-b/claude-5-haiku` | `claude-5-haiku` |
| Model B2, `bdd-provider-b`'s own default_model | `bdd-provider-b/claude-5-sonnet` | `claude-5-sonnet` |
| Model C, the conversation override | `bdd-provider-c/qwen-5-turbo` | `qwen-5-turbo` |
| Served id reported for B in the served-model case | `claude-5-haiku-20261101` | |
| Disabled choice | provider `bdd-provider-off` (enabled = false), model `off-model-1` | none |
| Unavailable choice | `bdd-provider-b/claude-5-retired` | none |
| Conversation X / Y | X has override C; Y has none | |
| Agents | `Default Assistant`, `BDD Agent Dawn` (no model), `BDD Agent Aurora` (model A), `BDD Agent Cirrus` (extension model C) | |
| Messages | unique per send, prefix `S1 `; reply text `bdd-reply-<served wire id>` | |
| Origin labels | `system default`, `this conversation`, `agent`, `configured fallback` | |
| Activation label for a default change | `live` | |

Why B differs from B2: the per-provider `Set as default` button sets the provider default with
its current default_model (B2). An oracle that expects B therefore cannot be satisfied by the
button; only the picker path (save model, then live route) serves B. The button stays, as a
legitimate provider-level action, and no S1 oracle accepts it as S1-R1.

Receipt time comparator (frozen in `fixtures/s1-oracle.json` `timeComparator`): second
precision, displayed `HH:MM:SS` in the browser's time zone; the receipt's change time must lie
inside the window the harness records around the change's confirm click and the live route's
acknowledgement, with 2 seconds of tolerance, and must not lie inside the window of any other
change; consecutive default changes are at least 5 seconds apart. For a turn already
generating, the expected time is the activation of the model it started with (A's), not B's.

### S1-R1. Live default
- Actor: authorized operator (deployment authority).
- Condition: system default is A; operator picks B in the system default picker and confirms.
- Outcome: the console saves B's model on the provider (`PUT`), then calls the live route (`POST .../default`), in that order; the confirmation appears only after the runtime acknowledged (observed by holding the live route and asserting nothing is confirmed while it is held), shows the requested value and the activation `live`; no restart; the next reply is served by B.
- Value: the operator changes what the whole system answers with, without restarting.
- Exclusions: no write to `llm.default_provider`, `llm.model` or `run_policy.global.model`; the per-provider button is not this action; no multi-replica or restart-survival claim in S1.
- Unknowns: how the console shows "acknowledged" is UNVERIFIED until implemented.

### S1-R2. Next eligible turn
- Actor: chat user in Conversation Y (no override).
- Condition: default changed A to B at time T; Y sends after T.
- Outcome: the reply is served by B; the transcript's receipt shows the model that actually answered as reported by the provider, the origin `system default`, and the time the default became that model (T, second precision, comparator above).
- Value: the person sees which model answered and why.
- Exclusions: the requested model is never shown as served unless it served; earlier messages keep their receipts.
- Unknowns: the receipt UI element; the time source (the `llm.default_provider` row's update time or the provider row's); the backfilled default has no system-default origin today (`PolicyScope` has Global, Agent, Conversation, Turn, Legacy only, `policy.rs:346-357`), so an origin must be added (task 7.1).

### S1-R3. Override holds
- Actor: chat user in Conversation X (override C).
- Condition: default changed A to B.
- Outcome: X's next reply is served by C; the effective view shows the requested value C, the effective value C and origin `this conversation`; the receipt says the same.
- Value: a choice made for one chat is not silently replaced by a system change.
- Exclusions: an override on an unavailable model is S1-R7.
- Unknowns: the effective view's layout.

### S1-R4. Clear to inherit
- Actor: chat user in Conversation X.
- Condition: X has override C; the system default is B; the user clears the override.
- Outcome: before confirming, the sheet previews the model the runtime would actually serve and its origin (`bdd-provider-b/claude-5-haiku`, `system default`); cancelling changes nothing; after confirming, X's next reply is served by B.
- Truthful fallback: if the default's provider is disabled the runtime serves the YAML fallback, so the preview names `openai/gpt-5.4-mini` with origin `configured fallback`, never the disabled default. This holds while the fallback exists; if the operator retires it (open decision 4) the preview must instead refuse the clear with the unavailable reason.
- Value: the person knows what clearing will do before doing it.
- Unknowns: today clearing sets an empty value with no preview (`session-config-panel.tsx:75-86`).

### S1-R5. Isolation
- Actor: chat user.
- Condition: with Y on the default, X changes its override C to another model, then clears it; Y sends after each step.
- Outcome: every Y reply is served by the default before, during and after; at the end, changing the default moves Y, which proves Y was observable throughout. X's changes alter no other conversation, agent or the global policy.
- Value: configuring one chat is safe.
- Exclusions: replica or tenant isolation.

### S1-R6. In-flight binding
- Actor: operator and chat user.
- Condition: Y has a turn whose stub reply is held (generating on A) when the operator, from a second page, changes the default to B.
- Outcome: after the change is acknowledged the reply is still held and the stop control still visible, proving the change landed mid-flight; after release the turn completes on A with one request, its receipt names A and shows A's activation time, not B's; Y's next message is served by B.
- Value: nobody's answer changes mid-sentence.
- Exclusions: cancellation; tool calls in flight.
- Unknowns: the stub needs a hold and release kind (tasks 7.10); no fixed delay is used, so there is no race.

### S1-R7. Truthful failure
- Actor: operator and chat user.
- Condition: the operator picks `bdd-provider-off/off-model-1` as the system default, or a user picks `bdd-provider-b/claude-5-retired` as an override.
- Outcome: refusal with a reason a person can act on (provider disabled; model not offered); the previous effective value stays and is shown; the registry default and the stored default provider read back unchanged; the next reply is served by the previous model; no request reaches any stub for the refused models; no receipt names them.
- Value: a failed change cannot be mistaken for a successful one.
- Exclusions: store outages and two-replica failure (SCN-SYS-006, supporting).
- Unknowns: the YAML fallback equals A, so a persisted-but-refused default could hide behind it; the scenario therefore reads the registry pair and stored row directly, and a sentinel `UAR_LLM__MODEL` is used when S1 runs in its own config (task 7.11).

### S1-R8. Agent pin
- Actor: operator and chat user.
- Condition: Aurora has model A, Cirrus has an extension model C, `Default Assistant` and the user-created Dawn have none; the default changes A to B.
- Outcome: Aurora stays on A and Cirrus on C; Dawn and `Default Assistant` follow B; the change result lists exactly {Aurora, Cirrus} as pinned and states a follow count equal to the other agents.
- Value: the operator learns that the change did not reach pinned agents.
- Exclusions: team members and conversation overrides are not listed; no lock-to-system option.

### S1-R9. Served model, not requested model (rule from memory record 02)
- Actor: chat user.
- Condition: the provider reports a serving model id different from the requested one (request `claude-5-haiku`, response `claude-5-haiku-20261101`).
- Outcome: the receipt names the reported id on the serving provider and never shows the requested candidate as the answering model.
- Value: the transcript is a record of what happened, not of what was asked.
- Unknowns: whether the driver exposes the response's model (task 7.12).

### Implementation implications (for Plan)
Tasks 7.1-7.12 in `tasks.md`: origin, time, receipt from the served model, effective view, clear preview, isolation guard, in-flight binding with a held-reply stub kind, truthful refusal, test support, stub extensions, per-provider stub instances and provider setup, served-model capture.

## S1 scenario set

One fixture vocabulary for the slice: the synthetic values above, frozen in
`fixtures/s1-oracle.json` and used by `tests/bdd/features/live-model-choice.feature` and
`live-model-choice-controls.feature`. SCN-SYS-001, 003 and 007 are expressed in that vocabulary.

For the S1 boundary run, the invalidation keys, binding check and repair rule in
`.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/adopted-contract.json` govern. The
per-scenario "Invalidation keys" lines in this document are design-time pointers to the code each
scenario exercises; they are narrower than the contract and never decide whether evidence carries
forward.

S1 mandatory means the scenario is in the S1 contract. Baseline is
`f2c9e36efe088e06a17c5d880ada877e25f2ad52`. A scenario that uses the operator-change step
(which drives the new picker) or a stub kind the pre-phase stub cannot parse is `NOT_RUNNABLE`
there; `FAIL` means the baseline runs it and the oracle rejects it.

| Rule | Scenarios that prove it | S1 mandatory | Expected baseline | Control scenarios (controls feature unless noted) |
|---|---|---|---|---|
| S1-R1 | SCN-SYS-001, SCN-SYS-007 | yes, yes | NOT_RUNNABLE, NOT_RUNNABLE | unknown provider (001); `CTL-SYS-001-HELD-500` failed acknowledgement (001); per-provider button (001); direct settings write (007) |
| S1-R2 | SCN-SYS-015 with SCN-SYS-001 | yes | NOT_RUNNABLE | mutant M2 (receipt time from send time) on main scenario 2 |
| S1-R3 | SCN-SYS-011 | yes | NOT_RUNNABLE | cleared override (011) |
| S1-R4 | SCN-SYS-012 | yes | FAIL | default's provider disabled, truthful fallback preview (012) |
| S1-R5 | SCN-SYS-013 | yes | NOT_RUNNABLE | two chats sharing one agent (013) |
| S1-R6 | SCN-SYS-014 | yes | NOT_RUNNABLE | change before sending (014) |
| S1-R7 | SCN-SYS-016 | yes | NOT_RUNNABLE | valid choice accepted (016) |
| S1-R8 | SCN-SYS-003 | yes | NOT_RUNNABLE | agents without a model not pinned (003) |
| S1-R9 | SCN-SYS-017 | yes | NOT_RUNNABLE | mutant M1 (receipt label from the request) |

### Mutants

A mutant is the candidate commit named in the implementation-complete inventory plus exactly
one patch file under `openspec/changes/uiux-system-model-live-default/fixtures/`. The patch's
sha256 is recorded in the inventory. The run refuses to start if the digest differs, or if
`git diff --name-only <candidate>..` on the mutant tree lists any file other than the patch's
single target. The mutant worktree is created under `~/.claude/worktrees/` from the candidate
commit (not from HEAD), with submodules initialised before building. Judgement is made from
the mutant's JSON reporter file, never from the exit status; a failure at any other step makes
the control INVALID, not a pass. The receipt code does not exist yet, so both patches are
authored and their digests frozen at the implementation-complete inventory, before the
boundary run (task 7.13). The independent critic who confirms both patches and their target
files records the inventory's sha256 as `inventorySha256` and its commit as `candidateCommit` in
`.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/review/s1/inventory-confirmation.json`;
the binding check that opens every boundary command refuses an inventory whose sha256 or commit
differs, which voids the run. After a repair the critic confirms again against the new
candidate commit.

| Mutant | Patch file | Changes only | Control for | Reporter | Expected on the mutant |
|---|---|---|---|---|---|
| M1 | `m1-receipt-from-request.patch` | the receipt's served-model data source reads the requested model | SCN-SYS-017 (and S1-R9) | `mutant-m1-receipt.json` | scenario "The receipt names the model the provider reported, not the one requested" fails at the step `the answer's receipt names the serving model "claude-5-haiku-20261101" on "bdd-provider-b"` with every earlier step passed, AND scenario "A conversation without an override follows the new default and says so" passes (M1 is otherwise healthy) |
| M2 | `m2-receipt-time-from-send.patch` | the receipt's change-time data source reads the turn's send time | SCN-SYS-015 | `mutant-m2-receipt.json` | scenario "A conversation without an override follows the new default and says so" fails at the step `the answer's receipt shows the time the system default became "openai/gpt-5.4-mini"` with every earlier step passed (the first message is sent at least 5 seconds after A became the default, outside A's window), AND scenario "A conversation override keeps answering on its own model" passes (M2 is otherwise healthy) |

SCN-SYS-015 owns the receipt's origin and activation time, so its control changes the time
source; SCN-SYS-017 owns the served-versus-requested label, so its control changes that
source. They are different properties and different mutants. Both expected outcomes are also
frozen in `fixtures/s1-oracle.json` `mutants`.

Not S1-mandatory but kept in the change: SCN-SYS-002, 004, 005, 006 (two replicas and a remote
store), 008, 009 (baseline PASS guard), 010.

### Supporting scenarios in other changes S1 depends on (by ID, not edited here)
- `uiux-settings-truthful-status-and-removals`: SCN-SET-003 (settings API rejects `llm.default_provider`).
- `uiux-model-identity-graph-ownership`: SCN-IDN-001 (provider-qualified Model identity under shared bare ids).
- `uiux-schema-controls-and-pickers`: SCN-PICK-002 (searchable provider-grouped picker).
Replica, boot-safety, team and local-first scenarios are not S1 dependencies.

### New scenario rows

Common to all new rows: real collaborators are the service process, scratch embedded store,
served frontend driven by `pnpm test:bdd` in a real browser, the provider and session routes and
per-provider `stub-llm` instances (`tests/bdd/fixtures/live-model-choice.json`, plus
`live-model-choice-new-kinds.json` through `STUB_LLM_EXTRA_FIXTURES`); fixtures are the frozen
S1 values; substituted boundary is the model vendor (stub-llm); restoration is stopping the
services and deleting the scratch root. Oracles read the serving provider's stub request log for
the served model and a store or registry read-back for stored values, never the system under
test's own display as truth; the display is asserted separately against those truths. Every row
is nondeterministic with `stochastic`: samples 3 run as `--repeat-each=3 --retries=0`, grader =
oracle holds and the negative control rejects in the sample, stoppingRule fixed N=3, all must
pass (no pass on retry). Likelihood statements are judgment from source, not measurement.

**SCN-SYS-011. A conversation override keeps answering after the default changes**
- Actor: chat user. Useful outcome: override C survives a default change A to B and says where it comes from. Rule S1-R3. Mandatory (S1): true.
- Source requirement: `openspec:session-configuration#An explicit model beats the system default`; `openspec:session-configuration#Each answer shows the model that served it, why and since when`
- Reason now: the slice's core promise is that a chosen model is the model that answers; the sheet shows only a selector and no origin (`session-config-panel.tsx:75-86`). Likelihood: medium that a user cannot tell, judgment.
- Production entry: Conversation X with override C chosen in the sheet; operator changes the default to B; X sends `S1 r3 override`.
- Oracle: observation = the `bdd-provider-c` stub's request for the message, the transcript reply, the receipt and the effective view; truth source = `fixtures/s1-oracle.json` SCN-SYS-011 (`qwen-5-turbo`, requested and effective C, origin `this conversation`); comparator = exact request count, suffix match, exact labels.
- Negative control: input = the same default change with X's override cleared first; expected rejection = the reply is served by `claude-5-haiku` (controls feature, cleared override).
- Expected baseline: NOT_RUNNABLE (operator step needs the picker).
- Limitations: one instance; stub vendor. Invalidation keys: `frontend/src/features/chat/session-config-panel.tsx`, `src/uar/domain/policy.rs`, `tests/bdd/fixtures/live-model-choice.json`, `fixtures/s1-oracle.json`. Owner tasks: 7.1, 7.3, 7.4, 7.9-7.11, 8.11.

**SCN-SYS-012. Clearing an override previews and then inherits**
- Actor: chat user. Useful outcome: the person sees what the chat will answer with before clearing and gets it afterwards. Rule S1-R4. Mandatory (S1): true.
- Source requirement: `openspec:session-configuration#Clearing a conversation override previews the inherited result`
- Reason now: clearing today is a silent empty value; the person cannot know what the chat will answer with. Likelihood: UNKNOWN.
- Production entry: X (override C, default B set through the route by the harness) opens the sheet, clears, reads the preview, cancels (reply from C), clears again, confirms (reply from B).
- Oracle: observation = preview text, the serving stub's request for each message and its reply text; truth source = `fixtures/s1-oracle.json` SCN-SYS-012; comparator = exact qualified id and origin in the preview, suffix match on the wire model.
- Negative control: input = `bdd-provider-b` disabled before the clear. The runtime then serves the YAML fallback (`registry.default_model()` is None for a disabled provider, `src/llm/registry.rs:555-557`; `resolve_default_model`, `src/uar/runtime/manager.rs:11323-11331`), so the truthful preview names `openai/gpt-5.4-mini` from `configured fallback`; expected rejection of a wrong build = a preview naming `bdd-provider-b/claude-5-haiku` or a reply from the `bdd-provider-b` stub.
- Expected baseline: FAIL (clear exists, no preview; only baseline-runnable steps).
- Limitations: holds while the fallback exists (open decision 4). Invalidation keys: `frontend/src/features/chat/session-config-panel.tsx`, `src/uar/api/discovery.rs`, `src/uar/runtime/manager.rs`, `fixtures/s1-oracle.json`. Owner tasks: 7.5, 7.8, 7.9, 8.12.

**SCN-SYS-013. Changing one conversation leaves another alone**
- Actor: chat user. Useful outcome: X's override changes and clearing never touch Y. Rule S1-R5. Mandatory (S1): true.
- Source requirement: `openspec:session-configuration#One conversation's model choice does not change another`
- Reason now: a shared-scope write would silently re-route other people's chats. Likelihood: low, judgment: conversation policy is stored per session; not measured.
- Production entry: default B throughout; X sets another model, Y sends; X clears, Y sends; the operator changes the default to A and Y sends.
- Oracle: observation = serving stub request per message; truth source = `fixtures/s1-oracle.json` SCN-SYS-013 (`sequence`); comparator = per-message wire model equality; the last step proves Y moves when the default moves.
- Negative control: input = X and Y both use the agent `Default Assistant` and X sets override C; expected rejection = Y still served by `claude-5-haiku`, the agent read-back shows no model, `GET /api/uar/settings/presentation-policy` `policy.model` unchanged; a build that writes the override to the agent or global scope fails.
- Expected baseline: NOT_RUNNABLE (the scenario's final step uses the picker; the routing property itself holds at baseline but is not exercised by a baseline-runnable scenario).
- Limitations: no cross-tenant claim; agent read-back route UNVERIFIED. Invalidation keys: `src/uar/api/discovery.rs`, `src/uar/domain/policy.rs`, `fixtures/s1-oracle.json`. Owner tasks: 7.6, 7.9, 8.13.

**SCN-SYS-014. A turn already being written finishes on its model**
- Actor: operator and chat user. Useful outcome: an in-flight reply is not moved to the new default. Rule S1-R6. Mandatory (S1): true.
- Source requirement: `openspec:session-configuration#A turn already generating finishes on the model it started with`
- Reason now: people read an answer as it streams. Likelihood: UNKNOWN; source suggests the model is resolved once per turn (`src/server.rs:7727-7728`), unproven.
- Production entry: Y sends `S1 r6 hold` (the stub of `openai` holds the reply); the operator changes the default to B from a second page; after the acknowledgement the harness checks the reply is still held and the stop control visible; it releases the reply; Y then sends `S1 r6 next`.
- Oracle: observation = stub held count, request count for the message, reply text, receipt model and time; truth source = `fixtures/s1-oracle.json` SCN-SYS-014 (held 1 after acknowledgement, one request, `bdd-reply-gpt-5.4-mini`, A's activation window not B's, next reply from `claude-5-haiku`); comparator = equality.
- Negative control: input = the same change made before sending; expected rejection = the message is served by B and the stub holds nothing.
- Expected baseline: NOT_RUNNABLE (needs the hold kind, absent from the pre-phase stub, and the picker).
- Limitations: tool calls in flight not covered. Invalidation keys: `src/bin/stub-llm.rs`, `src/uar/runtime/manager.rs`, `tests/bdd/fixtures/live-model-choice-new-kinds.json`. Owner tasks: 7.7, 7.10, 8.14.

**SCN-SYS-015. The transcript shows what answered, why and since when**
- Actor: chat user. Useful outcome: each assistant turn shows the serving model, its origin and, for a system default, when the default became that model. Rule S1-R2. Mandatory (S1): true.
- Source requirement: `openspec:session-configuration#Each answer shows the model that served it, why and since when`
- Reason now: without a receipt a changed default is invisible. Likelihood: UNKNOWN.
- Production entry: Y sends `S1 r2 before` and `S1 r2 after` around an operator change from A to B at least 5 seconds after A was set.
- Oracle: observation = receipt text of each turn; truth source = the serving stub request, the frozen origin label and the harness-recorded change windows (`timeComparator`); comparator = receipt contains the served qualified id, origin `system default`, and a time inside the window of the change to that model and outside the other window.
- Negative control: mutant M2 (receipt time read from the turn's send time) running scenario 2 of the main feature; expected FAIL at the first time step on M2 with scenario 3 passing there, PASS on the candidate (see Mutants). Neither SCN-SYS-016 nor SCN-SYS-017 is this scenario's control.
- Expected baseline: NOT_RUNNABLE (picker step; no receipt).
- Limitations: time source UNVERIFIED (the harness window is the truth). Invalidation keys: `src/uar/domain/policy.rs`, `src/uar/runtime/manager.rs`, `frontend/src/features/chat/**`, `fixtures/s1-oracle.json`. Owner tasks: 7.1, 7.2, 7.3, 8.15.

**SCN-SYS-016. A choice that cannot be served is refused and nothing is claimed**
- Actor: operator and chat user. Useful outcome: refusal with a reason, previous value kept and shown, no false claim. Rule S1-R7. Mandatory (S1): true.
- Source requirement: `openspec:session-configuration#A model choice that cannot be served is refused with a reason`; `openspec:provider-model-settings-certification#The default route rejects a disabled provider`
- Reason now: the default route accepts a disabled provider today (`src/uar/api/providers.rs:367-399` checks existence only). Likelihood: high for the default route, observed in source; UNKNOWN for the override path.
- Production entry: pick `bdd-provider-off/off-model-1` as the default; pick `bdd-provider-b/claude-5-retired` as X's override; send.
- Oracle: observation = refusal text, displayed effective value, the registry pair and stored default provider read back, every stub log; truth source = `fixtures/s1-oracle.json` SCN-SYS-016 (reasons contain `disabled` and `not offered`; registry `default_id` `openai`; stored default `openai`; no request with `off-model-1` or `claude-5-retired` on any stub); comparator = keyword match and equality.
- Negative control: input = a valid choice (model C as override); expected = accepted and served by the `bdd-provider-c` stub, so refusals are not blanket.
- Expected baseline: NOT_RUNNABLE (the scenario starts with the picker step).
- Limitations: store outage is SCN-SYS-006; the YAML fallback equals A (read-backs carry the check). Invalidation keys: `src/uar/api/providers.rs`, `frontend/src/features/chat/session-config-panel.tsx`, `fixtures/s1-oracle.json`. Owner tasks: 1.7, 7.8, 7.11, 8.16.

**SCN-SYS-017. The receipt names the model the provider reported**
- Actor: chat user. Useful outcome: the transcript is a record of what served, not of what was requested. Rule S1-R9. Mandatory (S1): true.
- Source requirement: `openspec:session-configuration#Each answer shows the model that served it, why and since when`
- Reason now: the review found no S1 input where requested and served differ; the stub echoes the request model otherwise. Likelihood: UNKNOWN.
- Production entry: with B as the default, Y sends `S1 served`; the `bdd-provider-b` stub's fixture asks for `claude-5-haiku` and answers with model `claude-5-haiku-20261101`.
- Oracle: observation = the stub request (requested model), the receipt's model field and provider; truth source = `fixtures/s1-oracle.json` SCN-SYS-017; comparator = receipt model field equals `claude-5-haiku-20261101` exactly and is not `claude-5-haiku`.
- Negative control: mutant M1 (receipt served-model source reads the requested model); expected FAIL at the serving-model step with every earlier step passed, and scenario 2 passing on M1 (see Mutants).
- Expected baseline: NOT_RUNNABLE (needs `served_model` and the capability probe; no receipt).
- Limitations: depends on the driver exposing the response model (task 7.12, UNVERIFIED). Invalidation keys: `src/llm/liter_driver.rs`, `src/uar/runtime/manager.rs`, `frontend/src/features/chat/**`, `tests/bdd/fixtures/live-model-choice-new-kinds.json`. Owner tasks: 7.3, 7.10, 7.12, 8.17.

## S1 step phrases

Reused as they are (`tests/bdd/steps/common.steps.ts`, `agents.steps.ts`): `I send the message {string}`, `I switch the active agent to {string}`, `a fresh conversation using {string}`. The old last-request step `the outgoing request's model field is {string}` is not used by S1: it reads only the last logged request and passes on a failed turn.

Step contract: `the operator changes the system default to {string}` always drives the system default picker (save the model, then the live route) and records the harness-clock window around the confirm click and the live route's acknowledgement; the per-provider Set as default button does not satisfy it. The held-route steps split that operator step so the confirmation can be observed before the acknowledgement. `the operator confirms the system default change to {string} while the acknowledgement is held` attaches a page response listener that records the status of `PUT /api/uar/providers/<target provider id>` and arms `page.route` on `POST /api/uar/providers/*/default`, both before the confirm click; it waits until the route handler has intercepted a POST whose URL path is exactly `/api/uar/providers/<target provider id>/default` (the live route carries no body: `providers-api.ts:71`, `providers.rs:367-370`) and the recorded PUT status is 2xx; it then keeps the POST held for 2 seconds (`holdWindowMs` in `fixtures/s1-oracle.json`), polling at most every 100 ms that no confirmation is visible (any appearance fails the step), and returns at the end of that window. It never waits for a response to the held POST. The absence assertions that follow (no confirmation, the control does not show the new pair, a registry GET still returns the old default) therefore run at the end of the hold, immediately before release, while the POST is known to be pending; a console that confirms after sending the POST without awaiting its answer is caught during the hold. This rejects a UI that confirms after the PUT returns but before, or without, the POST. After `the held acknowledgement fails with status {int}` later matching requests pass through to the server. `the system default model is {string}` is a harness step that sets the default through the existing provider routes and records its window; it is baseline-runnable.

New phrases for the execute stage (not written now):
- Setup: `the stub providers {string}, {string} and {string} are configured with models {string}, {string} and {string}` (each provider gets its own stub instance; `bdd-provider-b` is PUT with models `claude-5-sonnet, claude-5-haiku` and default_model `claude-5-sonnet`); `the system default model is {string}`; `{int} seconds have passed since the system default was set`; `the stub supports {string}` (reads `GET /_stub/capabilities`; fails setup on the pre-phase stub); `the provider {string} is disabled`; `a conversation {string} with no model override` / `a conversation {string} with the model override {string}`; `the conversation {string} and the conversation {string} both use the agent {string}`.
- Operator actions: `the operator changes the system default to {string}` (waits for the acknowledgement); `the operator confirms the system default change to {string} while the acknowledgement is held` (selects the pair in the picker and confirms; the harness holds `POST /api/uar/providers/*/default` with Playwright `page.route` and does not wait); `the acknowledgement is released` (route.continue); `the console issued no write to the server after the failed acknowledgement` (3-second window, excluding the PUT already made); `the held acknowledgement fails with status {int}` (route.fulfill; later requests to the route are passed through); `the operator changes the system default to {string} from another page`; `the operator tries to change the system default to {string}`; `a direct request sets the default provider to {string}`; `a client writes the setting {string} as {string} through the settings API`; `I use the per-provider Set as default button for {string}`.
- Conversation actions: `I switch to the conversation {string}`; `I switch to a fresh conversation using {string}`; `I clear the model override of this conversation`; `I set the model override of this conversation to {string}`; `I try to set the model override of this conversation to {string}`; `I confirm the change` / `I cancel the change`; `I send the message {string} without waiting for the reply`.
- Agents: `an agent {string} with the model {string} exists`; `an agent {string} with the extension model {string} exists`; `an agent {string} with no model exists`.
- Served-model checks (select the one stub request whose last user message equals the message, require exactly one, and the reply text `bdd-reply-<wire id>` visible): `the reply to {string} came from {string}`; `the reply to {string} did not come from {string}`; `the reply to {string} came from {string} with exactly one request`; `the stub of {string} received one request for {string} asking for the model {string}`; `no outgoing request used the model {string}` (searches every stub log).
- In flight: `the stub for {string} is holding the reply to {string}`; `the reply to {string} is still held and the stop control is still visible`; `the held reply is released`; `the reply to {string} finishes with content containing {string}`; `the stub holds no reply`.
- Receipts and views: `the answer's receipt names {string} and the origin {string}`; `the answer's receipt shows the time the system default became {string}`; `the answer's receipt does not show the time the system default became {string}`; `the answer's receipt names the serving model {string} on {string}`; `the answer's receipt does not name {string} as the serving model`; `no answer's receipt names {string}`; `the {string} conversation shows the effective model {string} from {string}`; `the {string} conversation shows the requested model {string} and the effective model {string} from {string}`.
- Preview and results: `the preview says this conversation will answer with {string} from {string}`; `the preview does not name {string}`; ``the confirmation shows the requested value {string} and the activation {string}`; `no confirmation is shown yet` / `no confirmation is shown`; `the system default control does not yet show {string} as live` / `the system default control shows {string} as live` / `the system default control still shows {string} as live`; `an error says the system default was not changed`; `the change is refused with a reason mentioning {string}`; `the change result lists exactly {string} and {string} as pinned`; `the change result does not list {string} as pinned`; `the change result's follow count equals the number of other agents`.
- Read-backs: `the registry default is still {string}`; `the stored default provider is still {string}`; `the settings API refuses the write with a message pointing to the live default route`; `the agent {string} still has no model of its own`; `the global policy still has no model`; `the console's writes to the server were, in order, {string} and {string}`; `the console wrote none of the settings {string}, {string} and {string}`.

Fixture and support needs: `tests/bdd/fixtures/live-model-choice.json` (cross product of every S1 message and the models `gpt-5.4-mini`, `claude-5-haiku`, `claude-5-sonnet`, `qwen-5-turbo`, with and without tools) and `live-model-choice-new-kinds.json` (the hold and served-model entries of `fixtures/s1-oracle.json`) loaded by the S1 stub instances; per-provider stub ports; providers `bdd-provider-b` (reconfigured), `bdd-provider-c` and `bdd-provider-off` through `tests/bdd/support/api.ts`.

UNVERIFIED for S1: the receipt UI element and origin source; the time source; the sheet's clear and preview control; whether the Session Configuration override reaches the turn on the current tree (a `tests/bdd/support/world.ts` comment calls the sheet's POST a dead facade; `6d3bc546` repaired the entity flow); the stub extensions; whether the driver exposes the provider response's model.
