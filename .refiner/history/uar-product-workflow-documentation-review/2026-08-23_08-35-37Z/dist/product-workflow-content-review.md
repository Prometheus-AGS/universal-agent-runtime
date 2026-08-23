# UAR product-workflow content review

## Scope

This review covers the seven source guides declared in
`docs/publication/product-workflows.json` and the concise `/docs/skills`
compatibility page. It is a `direct:content` source review. It does not certify
the production build, rendered diagrams, browser behavior, accessibility,
deployment, fresh runtime execution, or any runtime profile.

## Constraint evaluation

| Constraint | Result | Observed evidence | Limit |
|---|---|---|---|
| Complete classified workflows | Satisfied | The direct validator reported `Documentation product-workflow validation passed (7 guides)`. Missing-guide and unclassified-record controls failed for their intended mutations before the complete fixture passed. | Source files, manifest, classification, frontmatter, headings, markers, and links only. |
| Truthful inference evidence | Satisfied | Provider pages separate catalog metadata, configured availability, and genuine execution. The synthetic-as-genuine control failed. Retained summaries name provider/model, packaged boundary, 2026-08-22, source SHA, `server-full`, and a non-transfer limit. | Reviewed retained evidence; no fresh inference on this checkout. |
| Safe skill lifecycle | Satisfied | The skill guide records built-in, configuration-provisioned, and API-created provenance; conversation-over-agent-over-global precedence; next-request binding; restart durability; tombstone/restore; and the built-in/API exclusion. Removing the non-destructive safety marker made the control fail. | Documentation/spec correspondence only; no runtime skill operation was rerun. |
| Separate knowledge and memory authorities | Satisfied | The guides distinguish knowledge documents/chunks/citations from memory records, selected context, live events, and process-local history. Replacing the explicit boundary made the conflation control fail. | Source semantics and current authorities only; no ingestion or recall was rerun. |
| Public-safe profile-bounded source | Satisfied | Profile/state, missing-diagram explanation, and unsafe-private-excerpt controls failed as intended. TypeScript, architecture, brand, product-workflow, and publication control suites exited 0. The manual/automated safety audit found no machine path, credential, private key, raw event/session payload, raw `.prometheus` path, `versions.toml` claim, or exact private-evidence copy. | No production build, browser, accessibility tree, visual comparison, search interaction, deployment, or cross-profile evidence. |

## Structural and technical review

- The guide chain is provider configuration → model selection → genuine
  inference → agents → skills → knowledge → memory.
- Provider API writes are distinguished from catalog discovery and external
  startup configuration.
- Agent editor state is distinguished from effective run policy and live run
  events; the embedded host boundary is separate from HTTP/UI behavior.
- Knowledge requires accepted upload, completed indexing, ranked retrieval,
  attachment, citation, and grounded model output before making the bounded
  inference claim.
- Memory is explicitly opt-in and requires successful service initialization;
  response completion alone is not auto-capture evidence.
- The existing `/docs/skills` route remains a short compatibility entry point
  and delegates lifecycle authority to `/docs/skills/overview`.

## Deterministic checks observed

- `npm --prefix website run typecheck` exited 0.
- Architecture, brand, product-workflow, and composed publication control suites
  exited 0; every named negative control printed `PASS` and each complete
  fixture printed `PASS`.
- `openspec validate document-inference-skills-knowledge-and-agents --strict`
  reported the change valid.
- Direct product-workflow validation reported seven passing guides.

## Regression review

No shared global navigation, branding source, or
`docs/publication/routes.json` change was introduced. No runtime, React
application, provider/model behavior, dependency, vendored source, README,
lockfile, raw `.prometheus`, or deployment workflow belongs to this content
change. The final scoped diff gate remains the authority for that claim before
commit.

## Convergence

All five blocking content constraints have observed source evidence, and the
negative controls demonstrate that the required failure modes are detected.
The bounded content review converges in one iteration. Final-site build,
browser/accessibility, deployment, fresh inference, runtime behavior, and
cross-profile claims remain explicitly deferred.
