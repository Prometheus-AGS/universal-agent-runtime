# AG-UI settlement repair — gate006 source evidence

Observed: gate006 AG-UI caller handoffs emitted caller functions followed by host_cleanup_unconfirmed. Its no-tools choice later hung after the provider had completed text output. Anthropic passed 19/19, so shared caller schema capture was not broadly absent. OpenAI mounted-route forwarding is owned by another writer.

Two source defects share a terminal/producer boundary:

1. Concurrent AG-UI background/public reconciliation both call the real producer join. After joining, await_settled_run_status re-probed try_lock through settled_run_status; another idempotent joiner could hold the slot, falsely revoking already-established settlement. The await helper now reads and requires the actual terminal run status after successful producer and graph joins. Join failures remain errors. Ordinary join panic marks Error before releasing the producer-slot lock, preventing another waiter from seeing an empty handle and stale Done.
2. Text-only reconciliation used the nonblocking settled status probe after RunDone. That frame precedes outer finalization; a None result led to waiting for a new event, but finalization does not publish another frame. Once a retained terminal frame is observed, AG-UI now checks exact authenticated run ownership and awaits the actual producer/graph join before projecting terminal output. Before any terminal frame, existing nonblocking behavior remains.

Changed only src/uar/runtime/manager.rs helper and src/uar/api/agui.rs reconciliation/diagnostic span. Cleanup confirmation still fails closed; no event, status, or completion is fabricated. The diagnostic logs only a fixed error category and run ID, never raw persistence/provider error text. Regression authoring requested from the acceptance owner: concurrent AG-UI observers receive the same settled handoff; text-only/choice none returns terminal after actual cleanup without waiting for a second event.

No tests, builds, probes, checks, imports, network calls, formatting, or runtime commands executed by this writer. Lead owns the functional fix/retest loop. Changes remain unverified.
