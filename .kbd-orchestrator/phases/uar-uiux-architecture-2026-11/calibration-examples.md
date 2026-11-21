# Calibration examples — uar-uiux-architecture-2026-11

Owner: the operator (decision owner). Status: **drafted by the lead from the operator's
own 2026-11-19 decisions; owner review is required at contract adoption.** The SHA-256 of
this file is bound in `adopted-contract.json` → `calibration.sha256`. Any edit changes the
digest and requires re-adoption.

Each example says what a correct outcome looks like, so an oracle can be checked against
it. Source = the operator decision in `.prometheus/decisions.md` (2026-11-19).

## Positive examples (this is success)

1. **System default changes the next turn everywhere.** The operator opens the console,
   picks a different provider and model as the system default from a picker, and the
   next reply in an already-open conversation whose agent names no model uses the new
   model. A second browser tab and a client connected to another replica show the new
   default without a reload. Source: system level = registry default; realtime incl.
   replicas.
2. **Agent pin wins.** An agent with an explicit model keeps answering on that model after
   the system default changes, and the console lists that agent as pinned. Source:
   agent wins.
3. **Team adopts at the boundary.** A running team's member model is changed; the task in
   flight finishes on the old model; the next task runs on the new model; the task record
   shows which model each task used. Source: next task boundary.
4. **Bad YAML does not brick.** The operator saves invalid YAML while UAR runs, then
   restarts; UAR starts from the last-known-good configuration and the console shows a
   visible warning naming the rejected file. Source: never bricking; DB wins.
5. **No typing for enumerable values.** Every provider, model, embedding model and
   enumerable setting is chosen from a picker; text entry appears only for prompts and
   content. Source: goals 6-7.

## Rejected examples (this is not success, even if a check passes)

1. A picker that writes `llm.default_provider` and only takes effect after restart while
   the console says "effective". Source: analysis delta 5.
2. Realtime verified only on embedded Surreal or a single process. Source: remote and
   replica decisions.
3. A team task silently switching to a substitute model when the newly selected route is
   unavailable. Source: next task boundary ("fail visibly").
4. A last-known-good fallback that boots on an older, weaker security configuration after
   a deliberate security refusal. Source: analysis unresolved finding 4.
5. Settings that no code reads still editable in the UI. Source: decision 6.

## Rare but critical examples

1. A YAML edit that tries to override an operator-edited DB value is shown as drift and
   never silently wins. Source: DB wins.
2. A replica restarted mid-change converges to the current DB state without a manual step.
   Source: replica decision.
