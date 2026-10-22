# A2A durable observation repair

Observed gate003 failure: shutdown retained `a2a-durable-projection:run-event-source-unavailable` after an A2A actor returned a durable failed task for pre-provider `budget_admission_failed`.

Source cause: `submit_authorized_prompt_with_client_tools` only enqueues the actor mailbox and returns its exact completion receiver. The durable projection immediately attempted manager.subscribe before the actor necessarily registered a RunManager event channel. Absence at that point was classified permanently as lost observation, although the actor completion remained available and could prove failure.

Changed only src/uar/api/a2a/thread_service.rs. The durable projection now retains the exact actor completion through a proxy receiver installed in Entry. It subscribes to manager registration notifications before looking for the run event channel, allows that channel to be absent during admission, and wakes on either registration, actual run events, committed work, or exact actor completion. A completion is forwarded into Entry's existing refresh path, preserving its durable failed/unknown/cleanup semantics. If Entry has already consumed completion, the existing authoritative task is persisted directly. Real event-source closure after subscription still fails the observation; unknown or closed actor completion still follows Entry's existing unconfirmed path. CAS conflict re-enters authoritative snapshot immediately rather than waiting for a new event after consuming terminal completion.

No auth, task identity/correlation, execution, budget, config, server, or manager changes. No tests, checks, builds, probes, formatting, or runtime commands performed. Lead owns the authorized local functional retest. The repair is source-authored and remains unverified.

## Corrective ownership repair after source review

The source review identified that the first repair extracted the original mailbox completion before lifecycle task admission and dropped it on observer cancellation. That could turn observer failure into a falsely closed Entry completion channel.

Extraction now happens inside the admitted observer future, after it confirms the exact Entry/run binding. Rejected lifecycle admission therefore leaves the original receiver untouched. The observation loop runs inside an inner future; on cancellation or any other loop exit, an unconsumed original receiver is restored to the exact still-owned Entry/ActorTurn run before the proxy sender is dropped. If the real actor completion was already consumed, its result has been forwarded through the proxy; actual actor channel closure retains the existing unconfirmed semantics. No manufactured completion or success fallback was added. Source-only correction; lead-owned corrective review/build/gate004 remain pending. No test, check, build, probe, format, or inventory command was run by this writer.
