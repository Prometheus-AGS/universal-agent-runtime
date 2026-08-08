# Verification: amend-goal4-base-ui-divergence

Date: 2026-08-07
Phase change: C-01

## Acceptance evidence

1. Phase Goal 4 names `Base UI-backed local wrappers` and states that D1 is an override of KnowMe §6.1/§6.3 rather than compliance with their shadcn requirement.
2. `docs/ui-design-authority.md` defines source precedence, links internally to a self-contained D1 rationale, records its control-plane provenance, limits the override to general controls, navigation, overlays, and sidebars, and preserves all unaffected ownership and design requirements.
3. `openspec validate amend-goal4-base-ui-divergence --strict` passes.
4. The scoped change contains documentation, OpenSpec, KBD-goal, and refinement artifacts only. It changes no runtime code, provider compatibility, realtime state, persistence, API, or dependency behavior.
5. The vendored KnowMe standard body and historical shadcn references remain untouched by C-01; the new authority page resolves them for active implementation without erasing history.

## Commands run

```text
openspec validate amend-goal4-base-ui-divergence --strict
rg -n 'Target stack|Base UI|shadcn|D1|§6\.1|§6\.3' \
  .kbd-orchestrator/phases/uar-uiux-full-migration-2026-08/goals.md \
  docs/ui-design-authority.md \
  openspec/changes/amend-goal4-base-ui-divergence
```

Both checks passed. No runtime test suite is applicable to this documentation-only reconciliation.

## Pre-existing authority evidence

The active phase control-plane files are untracked in this working tree, so Git renders the scoped `goals.md` input as a new file rather than a one-line amendment. C-01 did not author the other goal bullets. The exact pre-edit file is preserved at `evidence/goals-before.md`; comparing it with the live goal file changes only Goal 4. Before the edit, Goal 4 ended with:

```text
assistant-ui + shadcn restyled Flat 2.0
```

- Pre-edit goals SHA-256: `6c8d1c1a6338f3a66b2e86e477fe5deb67b54a6794a4dbf3df3469e69069083d`
- Mechanical comparison result: `PASS (one changed line: 6)`

The independently authored phase plan confirms both that precondition and C-01's exact scope:

```text
C-01 ... Amend Goal 4 to name Base UI; add the §6.1/§6.3 divergence to
docs/ui-design-authority.md citing D1. The vendored standard header is already amended.
...
Leaving "shadcn" in the goals makes 17 downstream changes read as off-spec.
```

- Plan SHA-256: `e1f38fcfc86b10f27241ed20dcc1338a3643e21cf1b6df6ab4cf2244b703f41f`
- D1 decision-log SHA-256: `4cdab1b2fd2250d09a9a138b6a5800e0be15a02cb7ad1741761f33642ff130f9`
- Vendored-standard SHA-256: `e28bf74c41457daffed8c2af20daae4771e65971d646e540b11b617a7dd76484`

The D1 record begins `Keep Base UI; amend Goal 4` and expressly calls the choice an operator override of §6.1 and §6.3. The vendored header expressly says UAR uses Base UI per D1, calls it an override rather than compliance, and preserves the other §6.3 ownership rows. Both files existed before C-01 and remain outside its scoped file list.

The public authority page now links internally from its decision index to the D1 rationale. The KBD decision-log path is preserved as plaintext provenance rather than as a runtime documentation dependency.

Future public design divergences follow the same portability rule: reproduce scope and rationale on the authority page and retain KBD state only as audit provenance.

## Adversarial review remediation

Round 1 was isolated through the local OpenAI-compatible gateway with producer `openai/gpt-5` and judge `k3` (`cross_model_check: verified-distinct`). Its anti-theater screen passed at score `0.0`. The single critical finding identified this checklist task as open while its underlying strict validation had already passed; the task is now checked after the review evidence was written. Two warnings about untracked prerequisite visibility are addressed by the hashes and excerpts above.

Round 2 identified that the public authority page's Markdown link to the untracked KBD decision log would not resolve in a documentation checkout. The page now reproduces D1's rationale, links to it internally, and retains the KBD path only as audit provenance.

The subsequent review identified that an untracked `goals.md` still prevented Git from demonstrating the one-line amendment. `evidence/goals-before.md` now preserves the full pre-edit baseline so the unchanged goal bullets and the single intended replacement are mechanically reviewable.

Strict validator output: `Change 'amend-goal4-base-ui-divergence' is valid`.

Normalized review artifacts are stored under `.kbd-orchestrator/phases/uar-uiux-full-migration-2026-08/review/amend-goal4-base-ui-divergence/`:

- Round 1 `findings.json` SHA-256: `73da04401363d923e1f930211197d3d8b1411f6d6fd8861c54e10db4785e0b3c`
- Round 2 `findings-round-2.json` SHA-256: `a592f00784e54b61a7b9cfff5588f80d9b9b1507e51123ebcece0760037f0a26`
- Round 3 `findings-final.json` SHA-256: `83055b3d388c6ae1297eb9b6f55eb833425ff69e836587ec5a7eaf6a2039e759`
- Round 4 `findings-closure.json` SHA-256: `d6a85ab58c1edaeee491bf27dd9054631c7e572db6f3e3addea218b0e38b1585`
