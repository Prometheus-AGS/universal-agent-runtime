# Publication contract review

## Decision

The foundation is fit to govern the remaining documentation changes. It is not
evidence that the portal is complete or publishable.

## Constraint results

| Constraint | Result | Evidence boundary |
|---|---|---|
| Source and route authority | Satisfied | Isolated controls reject unclassified, ambiguous, missing, duplicate, and unjustified exclusions. The current tree still reports planned documents that do not exist. |
| Private history boundary | Satisfied | Controls reject raw session/event shapes, local paths, secret-like assignments, private keys, and exact private-source copies without printing matched values. |
| Fail-closed composition | Satisfied | Child failures propagate; zero and two Pages publishers fail; one publisher passes in isolation. |
| Truthful incomplete baseline | Satisfied | The repository validator exits non-zero for the missing pages and competing publishers instead of weakening the contract. |
| Bounded change | Satisfied | Strict OpenSpec passes, superseded artifact hashes are preserved, and the scoped product-source diff is empty. |

## Review findings

- The source and route manifests divide ownership without creating a generated
  file-per-path inventory.
- Public history is constrained to reviewed synthesis with repository-relative
  provenance; private records remain version-controlled but non-public.
- The single entrypoint composes truth, publication, and workflow policy so a
  passing subset cannot be mistaken for publication readiness.
- The design's source-of-truth wording must acknowledge that `versions.toml` is
  absent in this checkout. No dependency or architecture claim may cite it as
  inspected evidence until it exists.
- The next implementation dependency is `repair-single-pages-portal`; content
  routes remain owned by their later registered changes.

## Uncomfortable fact

The current portal is intentionally red. Treating the validator's non-zero
result as a nuisance would restore the exact false-positive publication path
this contract exists to prevent.
