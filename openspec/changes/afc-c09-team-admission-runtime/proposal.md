# C09.3: Bounded team admission and execution

## Why

C09.1 persists team plans and C09.2 persists fenced task ownership and mailbox intent. Neither starts a member turn: the saved `assignmentAuthority` explicitly denies execution and tools. A team becomes usable for work only when UAR durably reserves its aggregate limits, resolves the current binding and host authority, then dispatches through the existing actor/thread runtime with effect-time revalidation.

## What changes

- Add one owner/workspace-scoped catalog transaction for a task attempt, aggregate budget reservation, fenced membership/ownership epoch and dispatch intent. Settle usage once per attempt/run identity; retain reservations when usage is uncertain.
- Dispatch admitted member turns through the existing actor-host/thread path. Recompute current Cedar, host, parent, binding and workspace constraints before a turn and before protected effects. A claim or mailbox `trigger-turn` intent alone grants nothing.
- Add revisioned member revocation, explicit context/artifact selection and scoped receipts. Never union private member history or grant a child broader access than the parent.
- Expose fixed authenticated REST and capability contracts for admission, execution status, cancellation/recovery, selected context/artifacts and usage. The Boss will surface these through its existing UAR Teams settings and typed IPC.

## Impact and dependencies

This is the initiative's C09.3 product increment; it depends on C09.1/C09.2. UAR remains execution owner; The Boss remains the trusted UI/host boundary. Existing ordinary runs and external protocol clients retain their behavior. The current durable `AgentInstanceController` supports local `surrealkv://` only, while the team catalog has remote Surreal support. This change must use a team-catalog-owned durable admission ledger and prove the selected remote path before advertising remote team execution. It must not silently fall back to local storage for an explicitly remote team.

The KBD task stays pending until a source-pinned Mac ARM64 application build and one installed-app integration scenario demonstrate a bounded member turn, protected-effect denial after revocation, reservation settlement and restart recovery. Windows x64 and Mac Apple Silicon public release remains a separate cadence publication obligation.
