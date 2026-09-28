# Design

The existing collaboration catalog CAS is the sole writer. Each mutation checks authenticated owner, workspace, team revision, task revision and materialized member identity. Claim and reassign increment `ownershipEpoch`; reviewer changes use a separate `reviewerEpoch`. Stale writers receive a conflict, and identical command replay returns the recorded result. Terminal failure/cancellation fences the old owner. No command transitions a task to running or succeeded in this increment.

Claim and reassign also persist `assignmentAuthority`, a non-executable projection of the exact binding, workspace, task role, selected member and epoch. It has `canExecute=false` and `canUseTools=false`; it is a narrowed planning constraint, not an effective Cedar or host grant. C09.3 must intersect current host limits, principal grants, parent constraints, binding and child request again before admitting a turn or effect.

An inbox message binds owner, workspace, team, recipient member and optional task ownership epoch. Public send is from the authenticated operator; clients cannot impersonate a member. Mode is the definition's `queue-only` or `trigger-turn`. Accepted means durable enqueue, delivered means recipient availability, processed means a later turn consumed it. Owner inspection does not alter delivery state. Trigger intent remains queued until C09.3 supplies aggregate budget and execution admission. The message and command receipt commit together. Replays check current disclosure scope.

The Boss will consume the fixed routes through its authenticated, workspace-scoped main process. Completed-boundary verification builds the actual Mac ARM64 installer and exercises claims, stale epochs, inbox receipt distinctions and restart through the packaged application.
