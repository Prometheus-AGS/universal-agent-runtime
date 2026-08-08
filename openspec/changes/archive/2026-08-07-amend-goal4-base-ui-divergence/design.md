## Context

The phase goals name the migration plan and delivered design artifacts as binding authority, but Goal 4 still assigns general frontend primitives to shadcn. Operator decision D1 retained the already-landed Base UI migration and explicitly classified that choice as an override of the KnowMe UI/UX standard §6.1 and §6.3. The vendored standard header records the exception, but there is no concise project authority page that tells implementers how to resolve this conflict.

This is a documentation and specification change only. It does not alter frontend behavior, provider compatibility, realtime state, persistence, or backend interfaces.

## Goals / Non-Goals

**Goals:**

- Make Goal 4 accurately name Base UI.
- Establish one canonical project page for frontend design-source precedence and recorded divergences.
- Scope D1 narrowly to the primitive-ownership rows it overrides.
- Preserve every unaffected KnowMe requirement, especially Assistant UI, PEM, Zustand, and PGlite ownership.

**Non-Goals:**

- Re-evaluate Base UI versus shadcn or claim an effort or quality advantage.
- Edit the vendored KnowMe standard body.
- Rewrite historical plans or ADRs.
- Change runtime code, dependencies, APIs, providers, or realtime behavior.

## Decisions

### Use a project authority page as the conflict-resolution index

`docs/ui-design-authority.md` will identify the binding phase sources, state how recorded operator decisions resolve conflicts, and carry D1's rationale with its control-plane provenance. An internal decision index links to the rationale so the public page remains usable when KBD working-state files are not distributed. This keeps the vendored standard intact while giving downstream changes one current entry point.

Alternative considered: rely only on the vendored header. Rejected because it records the exception but does not define project-wide precedence or explain how phase goals should stay aligned.

### Treat D1 as a narrow override, not standard compliance

The page and Goal 4 will say that Base UI-backed local wrappers own general controls, navigation, overlays, and sidebars for UAR. They will also state that this diverges from KnowMe §6.1 and the relevant §6.3 ownership row. All other standard requirements remain binding unless another recorded decision explicitly changes them.

Alternative considered: restate the standard as though Base UI satisfied its shadcn requirement. Rejected because D1 expressly classifies the choice as an operator override rather than compliance.

### Keep historical artifacts intact

Older documents may still describe shadcn. The authority page will make clear that current phase goals plus recorded decisions govern active implementation; historical references remain evidence of prior intent and are not silently rewritten.

Alternative considered: globally replace every shadcn reference. Rejected because it would erase historical context and expand this reconciliation change beyond its observed contradiction.

## Risks / Trade-offs

- **Risk:** Readers treat the Base UI override as permission to ignore the rest of the standard. → **Mitigation:** Name the exact overridden sections and explicitly preserve all other requirements and ownership rows.
- **Risk:** Historical documents appear contradictory. → **Mitigation:** Define current precedence and retain the dated D1 rationale and provenance on the public authority page.
- **Trade-off:** The same divergence appears in the vendored header and the authority page. → The duplication is intentional: the header protects readers entering through the vendored file, while the authority page governs the project.

## Migration Plan

1. Add the `frontend-design-authority` requirement delta.
2. Amend Goal 4 to name Base UI and classify the choice as the D1 override.
3. Add `docs/ui-design-authority.md` with precedence, scope, and unchanged requirements.
4. Validate the OpenSpec change strictly and record canonical KBD completion.

Rollback is a focused revert of the Goal 4 line, authority page, and OpenSpec change artifacts. No runtime or data migration is involved.

## Open Questions

None. D1 already resolves the component-primitive choice and its classification.
