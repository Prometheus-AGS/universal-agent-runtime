# A2A shutdown analysis — gate008

Analysis only. No production files edited and no commands executed beyond reading/searching source and existing evidence.

## Observed evidence

- candidate.log lines 2556–2557 show run-manager entering AdmissionClosed/Draining with exactly `a2a-durable-projection` operations 14:151, 14:155, and 14:179 outstanding.
- Matrix negative-provider scenarios report `A2A stream terminal task read failed`; this means the subsequent real GetTask response was rejected, not that its provider request succeeded. The two provider errors were genuinely exercised.
- The available candidate log does not map observer operation IDs to run IDs or expose their pending mutex/future stacks. No exact held-lock cause can be proven from this artifact.

## Source findings

The durable projection's CAS-conflict branch sets `first_snapshot = true` and immediately continues. Its cancellation select runs only when first_snapshot is false. Sustained conflicts therefore bypass shutdown cancellation between completed persistence attempts. A loop-top cancellation observation would allow the existing restoration epilogue to restore an unconsumed actor completion, without canceling an already accepted persistence operation. This finding was sent to the thread_service owner; no repair was made by this analyst.

The active A2A writer independently found stale Entry.durable revisions caused by another projection CAS and owns that correction plus completed replay. That can explain conflicts but does not itself prove which three jobs were still running at forced shutdown.

Examined completion/root lock ordering: ActorThreadSession.execute_request awaits kernel completion, joins root.finish, persists output and terminal thread, then returns. The agent actor releases its session mutex before sending the mailbox result. The observer restores the original receiver only after its inner Entry guard has left scope. These paths do not establish a producer/Entry lock cycle. No speculative change to root joins is warranted from the current evidence.

Negative-provider GetTask failures have no sufficiently specific host-cause log in the supplied artifact. They must remain unresolved until the lead-owned next functional gate or added bounded diagnostics identifies the exact failing operation; they must never be converted to success or ignored cleanup.

## Handoff

Thread-service owner received the CAS/cancellation finding. Lead owns consolidated build009 and local functional execution. All runtime cleanup guarantees and frozen image constraints remain unchanged. This is source/evidence analysis, not verified runtime acceptance.
