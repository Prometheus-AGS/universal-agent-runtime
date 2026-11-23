# UAR: a reachable VLBV exit

Start with the user’s job: choose a system or conversation model and know
which model actually answers; rename or delete a conversation from its menu
and see the change survive reload; start a new conversation and get its own
title. Report the behavior at the real UI/runtime boundary, not the number
of tests or commands executed.

## RC.6 evidence and the open phase are not the same thing

The focused RC.6 report at
`.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/rc6-check/3-bdd.json`
records 15 expected, 0 unexpected, 0 skipped and 0 flaky in 242.5 seconds.
It covers the `live-model-choice` and `thread-menu` files. Treat it as the
observed focused behavior run, not blanket certification of the signed
contract, container, or other platform.

At inspection, canonical phase `uar-uiux-architecture-2026-11` is OPEN.
The bound contract at plan revision 143 declares implementation incomplete
and contains 15 scenario IDs and 54 commands. The identical counts of
scenario IDs and focused browser specs do not establish one-to-one coverage:
the focused report contains a unique-title case and cancel controls, while the
bound criteria also name settings-route, provider-qualified identity, and
keyboard-picker checks. Reconcile any independently applicable supporting
evidence. Do not let a green command substitute for evidence of a predicate
that the command didn't exercise.

## Safe closeout

1. Freeze the exact candidate, not the whole working tree by implication.
   Map each still-required real predicate to current evidence using the
   scenario, source/build, configuration, and fixture/oracle identities.
   Preserve and keep separate the historical failed/blocked attempts.
2. Remove old baseline or deliberately broken rebuilds and repeat counts only
   through the existing signed amendment path, with the source/fixture/oracle
   changes honestly declared. Continue to check meaningful refusal, recovery,
   persistence and isolation on the real candidate. Keep an oracle where it
   protects the user's outcome; if no longer applicable, record why.
3. Use the existing narrow, authorized VLBV recovery for missing or affected
   predicates. Do not call a harness error a product failure; do not rerun
   unaffected valid receipts or an unchanged product merely to repair the
   harness. A command or receipt cannot make its own prerequisite true.
4. Close only through canonical VLBV when the evidence is admitted and
   required outcomes actually pass. The RC image's existing build, boot and
   commit/readback checks remain unchanged as release and artifact evidence;
   protected pushes, tags and publication are separate.

For future phases, reject a plan that cannot reach an exit on the intended
candidate. Do not add a second review or tier gate to close a phase.
