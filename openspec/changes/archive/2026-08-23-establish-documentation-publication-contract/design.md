## Context

See `proposal.md` for motivation and `specs/` for the observable contract. The repository already has an npm-managed Docusaurus site, a narrow documentation-truth script, a GitHub Actions policy validator, 38 tracked README files, hundreds of documentation and history files, and two Pages publishers. The design must classify the full estate without copying private history into the site, and it must remain usable by later documentation changes without adding a second metadata system.

The source of truth order remains unchanged: current implementation and the operator-owned `versions.toml`, when present, govern architecture and dependency claims; canonical OpenSpec governs behavior; KBD records lifecycle position; and `.prometheus` remains append-only evidence. `versions.toml` is absent from this checkout, so this change does not cite it as inspected evidence. Documentation manifests describe how those sources may be published; they do not supersede them.

## Goals / Non-Goals

**Goals:**

- Make the disposition of every tracked README, documentation, KBD, OpenSpec, ADR, and `.prometheus` path deterministic and locally verifiable.
- Give later portal changes one stable route and provenance contract.
- Fail closed when private history, machine-local data, secret-like material, stale claims, missing routes, or competing Pages publishers would be published.
- Preserve historical evidence while separating it from current public authority.
- Extend the existing Node validation entrypoints without introducing runtime dependencies.

**Non-Goals:**

- This change does not write the final portal content, alter the Docusaurus theme, repair the Pages workflow, or deploy the site; later registered changes own those slices.
- It does not change Rust runtime, React application, provider/model, or realtime behavior.
- It does not rewrite vendored documentation or publish raw `.prometheus`, KBD event, session, conversation, or wiki records.
- It does not add a documentation container, hosted search, analytics, or a second publication target.

## Decisions

### 1. Keep publication authority under `docs/publication/`

The change introduces three reviewed files:

- `docs/publication/sources.json` — classification rules and explicit overrides.
- `docs/publication/routes.json` — supported product surfaces, canonical document IDs/routes, source authority, profiles, and exclusions.
- `docs/publication/README.md` — contributor-facing explanation of the schemas, ownership rules, and local commands.

JSON is chosen because the existing validators are dependency-free Node scripts and can parse it without another package. A single combined manifest was rejected because source ownership and public navigation change at different rates and require different reviewers.

### 2. Expand classification rules against the tracked tree

`sources.json` uses non-overlapping rules with explicit path globs and optional exact-path overrides. Every rule records:

- stable ID and matched paths;
- disposition (`public`, `public-normalize`, `private-synthesis-only`, or `excluded`);
- owner and `current` or `historical` status;
- canonical authority;
- public destination when publishable;
- generation source when the matched file is a mirror;
- exclusion or synthesis rationale when not directly publishable.

`scripts/validate-documentation-publication.mjs` obtains tracked paths from `git ls-files`, selects README files and the declared documentation/history roots, and requires every selected path to match exactly one rule. Zero matches and multiple matches both fail. The validator prints resolved counts per disposition and every failing path, so broad rules remain auditable without committing a second generated inventory.

An explicit path-per-record manifest was considered. It would produce hundreds of repetitive entries and would be stale after every generated KBD record. Non-overlapping rules give the same exact resolved disposition while making additions fail until their ownership class is intentional.

### 3. Treat route coverage as a product-surface join

`routes.json` maps stable product-surface IDs from `docs/product-surface-inventory.md` to Docusaurus document IDs and expected public routes. Each entry records source files, supported profiles, status (`required` or `excluded`), and the reason for any exclusion. The validator requires every inventoried surface exactly once and requires every required document ID to exist under `website/docs/`.

Routes use Docusaurus document IDs rather than inferring URLs from filenames. The final site validator resolves those IDs through the built artifact and the deployed base URL. Filename-only discovery was rejected because explicit slugs and category indexes can change public URLs without changing paths.

### 4. Use provenance front matter for public history synthesis

Public architecture and testing history pages carry a `source_records` front-matter list of repository-relative ADR, OpenSpec, KBD, or `.prometheus` paths plus a `current_authority` link. Only reviewed prose is copied into page bodies. The validator confirms every referenced record exists and is classified `private-synthesis-only`, `public-normalize`, or historical public material; it rejects provenance pointing to missing or excluded third-party material.

The design deliberately does not generate history prose from logs. Automatic summarization would make invented rationale difficult to distinguish from observed decisions. Later history changes write reviewed synthesis and retain source pointers.

### 5. Put privacy and truth checks in one fail-closed local validator

`scripts/validate-documentation-publication.mjs` composes these checks:

1. manifest schema and exact source classification;
2. route-manifest coverage and document existence;
3. current/historical banner and authority rules;
4. provenance path validity;
5. sanitizer scanning of public Markdown, site assets, and an optional built-output path;
6. one-Pages-publisher policy by delegating to the existing Actions validator;
7. negative-control fixtures that must be rejected.

The sanitizer rejects raw history markers, `/Users/` and equivalent machine-local paths, credential/private-key shapes, copied event/session payloads, and private-source file bodies. It reports rule, file, and location without printing matched secret values. An allowlist is limited to fixtures and explanatory prose that names a prohibited pattern without containing a secret-shaped value.

Separate validators were considered but rejected for the public contract because a green subset could be mistaken for publication readiness. Existing specialized scripts remain callable, while the new entrypoint provides the complete contract result after documentation implementation is complete.

### 6. Preserve local-only verification and deployment-only Actions

The root package scripts gain a local documentation-publication command in this change. Later changes add the frozen Docusaurus build, link, responsive, keyboard, and accessibility stages to the same final local entrypoint. `.github/workflows/docs.yml` is not modified by this foundational change.

`scripts/validate-github-actions-policy.mjs` is tightened here to require exactly one Pages publisher and to stop allowlisting `typescript-sdk-docs.yml` as a separate publisher. The actual workflow consolidation and deletion occur in `repair-single-pages-portal`; until then the new validator is expected to report the observed collision. That known failing result is recorded as an implementation dependency, not mislabeled as a passing check.

### 7. Supersede the earlier portal change without rewriting its history

`openspec/changes/docs-hosted-rustdoc-typedoc-docusaurus-ia/superseded.md` records the successor phase/change, the conflicting requirements, and the disposition of its three open operator/follow-up tasks. The old proposal, design, task history, and timestamps remain intact. Canonical specs are changed by the current delta; the old change is not archived into them again.

This avoids two bad alternatives: silently editing the old evidence to look current, or archiving it and reapplying obsolete GitHub Actions testing and placeholder requirements.

### 8. Keep KBD lifecycle state canonical

After strict OpenSpec validation and implementation evidence, the change is transitioned through `prometheus kbd`; generated waypoint/progress projections are refreshed by the runtime. No generated JSON projection is edited manually. The next registered phase change remains `repair-single-pages-portal`.

## Risks / Trade-offs

- **[Risk] Broad classification rules could hide an incorrectly owned file** → Rules must be non-overlapping, use narrow roots, name an owner/authority, and include exact overrides for generated mirrors, vendored paths, and public synthesis.
- **[Risk] Secret-pattern scanning can expose or falsely flag sensitive text** → Report only the rule and location, never the matched value; keep narrow documented exceptions and require rejecting negative-control fixtures.
- **[Risk] This contract initially makes the current two-publisher tree fail** → Record the failure as the observed baseline and make `repair-single-pages-portal` the immediate dependent change; do not weaken the validator to preserve a false green result.
- **[Risk] Explicit route manifests can drift from Docusaurus routing** → Key entries by document ID, validate source existence locally, and resolve representative built routes in the final site gate.
- **[Risk] Hundreds of historical files could overwhelm public navigation** → Classify raw history as private-synthesis-only and publish curated chronological synthesis, not a file mirror.
- **[Trade-off] Rules are less visually exhaustive than one JSON record per path** → The validator's resolved counts and exact unmatched/ambiguous failures provide complete evidence without permanent generated churn.
- **[Trade-off] The earlier portal change remains unarchived** → Its explicit supersession record prevents it from acting as current authority while avoiding an archive operation that would reapply obsolete deltas.

## Migration Plan

1. Add the publication manifests, contributor guide, validator, and negative-control fixtures.
2. Record the earlier portal change's supersession without altering its historical artifacts.
3. Run the validator against the current tree and retain the expected competing-publisher failure.
4. Complete `repair-single-pages-portal`; rerun the contract until classification, privacy, route, and single-publisher checks pass.
5. Let later content changes populate all required routes and provenance records; the final certification change runs the complete local gate after code/content completion.
6. If the contract must be rolled back before dependent changes land, remove the new manifests/validator and revert only this change. Once dependent content uses the manifests, rollback requires reverting those dependents in reverse order rather than leaving ungoverned public content.
