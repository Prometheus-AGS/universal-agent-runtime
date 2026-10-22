# Source-bound cumulative token-budget recipe

Phase Execute, `uar-post-lifecycle-verification-readiness`. Source inspection and this documentation write only. No tests, scripts, imports, checks, probes, builds or network requests ran. Shared checkout preserved; no production files edited. Lead owns execution under actual operator authorization.

## Numeric recipe for the current ordinary server path

Use a dedicated persisted ordinary agent selecting **openai/gpt-4o-mini**, with the existing provider endpoint explicitly redirected to the isolated loopback peer, real protected verification completed, and only `extensions.budgets.max_tokens_per_session = 144386` set for this proof. Leave per-turn token, cost, rate and tool ceilings absent so another limit does not explain rejection. Preserve agent/config/owner/store through cold continuation.

The checked-in catalog gives context_window=128000 and max_output=16384 for this exact provider/model. build.rs copies catalog/provider_catalog.json into the embedded catalog. ModelCallBudget::begin_call in cost_budget.rs:1804 reserves catalog context+max_output when request.budget_contract is absent. RunManager initializes destination_preparations=None; the repository-wide with_destination_preparations search finds its definition and conditional forwarding, but no production startup installation. The ordinary configured server therefore takes this catalog branch, rather than reserving only the fixture's reported usage or shrinking its reservation to Anthropic max_tokens.

For that branch:

| Quantity | Tokens |
| --- | ---: |
| R1, first admission | 144384 |
| R2, resumed admission | 144384 |
| L, configured session ceiling | 144386 |
| S, first settled wire usage (input 2 + output 3) | 5 |
| S + R2 | 144389 |

Thus R1<=L, R2<=L and S+R2>L. A reset session would admit request two, so rejection distinguishes retained cumulative charge. The existing small usage suffices; no inflated usage or private ledger writes are needed. If the fixture's new explicit wire-usage control is used, set input_tokens=2/output_tokens=3 and retain its actual response/trace. This is controlled provider-reported usage, not a claim that the textual prompt literally tokenizes to two tokens.

Output contract: catalog max_output is 16384; the current Anthropic acceptance request explicitly caps output at 256. Reported output 3 is below both. Keep the source-derived caller cap unchanged across continuation. For any other protocol whose request is given an explicit cap, require cap>=3; do not choose a large synthetic output exceeding the actual cap. Catalog reservation remains 144384 without a budget contract even when a smaller request output cap is supplied.

## What must be observed in the lead gate

1. Retain exact candidate/catalog/config/agent identities and protected model-verification receipt. Model identity must really be openai/gpt-4o-mini; a custom fixture provider does not inherit this reservation or pricing.
2. First caller handoff succeeds with one actual provider contact, complete usage 2+3, terminal provider Done, and no accounting-unresolved receipt. Cost_budget.rs settles durable usage only after Done with reported usage and no stream failure. Accounting settlement replaces reservation with observed tokens; the token admission rule rejects only when settled+reserved+new reservation **>** L, so equality remains admissible.
3. Submit exact pending results in the same session. Require budget admission/exhaustion failure before a second provider connection. Since R2<=L independently, this tests retained prior charge. Preserve the provider trace and initial pending native session identity on each of the five interfaces.
4. Repeat the exact rejected batch and require no new provider contact. For cold proof, seed pending results, cleanly join candidate, restart identical binary/config/store/owner with peer still alive, then submit and replay the same batch. The process/join receipts establish an actual restart.
5. Retain the original failed diagnostic: cost_budget.rs admission refusal logs reservation_source, reserved_tokens and configured limits. Expected source is catalog_context_and_output and reserved_tokens=144384. A different reservation source, unresolved accounting, different model, or admission failure before first handoff invalidates this numerical proof; do not relabel it cumulative success.

## Conditional alternative, not the current numeric proof

If a future production setup installs a final-wire budget contract, the constant catalog calculation no longer applies. request_budget_bound computes input from validated serialized bytes plus count uncertainty and output from contract output+additional reasoning. ResolvedStep clamps contract output by the caller cap. Then obtain actual R1/R2 for each protocol/history and choose L>=max(R1,R2) with positive S>L-R2, while keeping reported input/output within that request's admitted bounds. Never reuse 144386 merely because the model name is unchanged. Existing admission-failure logs expose bounds on failure, not a successful-call reservation receipt; source identity and active branch are necessary evidence here.

Source receipts: inspected remaining-plan budget section; cost_budget.rs begin_call and stream settlement; persistence/accounting.rs reserve/settlement; liter_driver.rs request_budget_bound/enforce_request_budget; manager.rs preparation initialization/forwarding; turn/resolved.rs output clamping; acceptance driver Anthropic cap; fixture actual usage serialization; catalog model via bounded jq source selection; build.rs catalog copy. A broad initial rg on the one-line catalog produced truncated output, so the exact model was subsequently selected with jq. No execution acceptance is claimed.

## 2026-10-22 catalogue reservation source audit
Source-only audit of cost_budget.rs::begin_call confirms the ordinary catalogue branch reserves context_window plus max_output and does not read request.output_token_cap. The final-wire branch is selected only with request.budget_contract Some. The isolated ordinary openai/gpt-4o-mini budget agent has no execution profile/destination preparations, so the authored proof remains R1=R2=128000+16384=144384, settled usageS=2+3=5 andL=144386: R<=L andS+R>L. Anthropic max_tokens256 does not narrow that catalogue reservation. No edit to limits or caps is justified. Actual branch/reservation observations remain for the authorized gate; a final-wire contract would invalidate this numeric proof. No verification ran.
