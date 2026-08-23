## Context

See `proposal.md` for motivation and `specs/dev-portal-2026/spec.md` for the
observable contract. The existing Docusaurus site has a pinned npm lockfile,
but its `npm run build` path shells out to pnpm. The main Pages workflow calls a
nonexistent TypeScript script and creates placeholder output when generation
fails, while a second workflow publishes the TypeScript artifact directly to
the same Pages environment.

Docusaurus' current deployment guidance confirms that an npm-managed Pages
build installs with `npm ci`, runs the production build, uploads the resulting
`build/` directory, and deploys it through the Pages environment. UAR retains
its existing project-site `url`, `baseUrl`, `trailingSlash`, and `.nojekyll`
settings.

## Goals / Non-Goals

**Goals:**

- Make one reproducible npm command build the narrative portal.
- Stage genuine rustdoc and TypeDoc outputs into that one portal artifact.
- Make absent or malformed generated references stop artifact assembly.
- Leave exactly one Pages publisher with post-deployment route validation.
- Keep routine tests, linting, accessibility, and conformance out of Actions.

**Non-Goals:**

- This change does not rebrand the site, write the missing product guides, or
  make the final publication-readiness claim.
- It does not add Python reference generation because no pinned Sphinx build
  contract exists in the repository.
- It does not change runtime, frontend application, provider, inference, or
  realtime behavior.

## Decisions

### 1. Keep npm as the site-local package authority

Change the Docusaurus build script to `npm run copy:adr && docusaurus build`.
The copy step remains dependency-free Node code, so local and deployment builds
use the same checked-in command on macOS and Linux.

Using the repository-wide pnpm workspace was rejected because the site already
has a dedicated npm lockfile and the observed failure is the build crossing
package-manager boundaries after a successful `npm ci`.

### 2. Generate references before assembling the final artifact

The sole workflow builds Docusaurus, generates workspace rustdoc, installs the
TypeScript SDK from its own npm lockfile, and runs its real `npm run docs`
command. A dependency-free staging script then requires each generated
`index.html` and copies the complete reference trees to
`website/build/docs/api/{rust,typescript}`.

Direct shell copies with `|| true` and generated placeholder HTML were rejected
because they turn missing reference generation into a green deployment.

### 3. Remove, rather than disable, the competing publisher

Delete `.github/workflows/typescript-sdk-docs.yml`. Keeping a disabled or
manually triggered publisher would still create a second owner that can later
drift or overwrite the full portal.

### 4. Keep Actions deployment-only

The workflow may install documentation dependencies, generate the publication
artifact, upload it, deploy it, and request deployed routes. It does not run
prose lint, source validation, unit/integration tests, typechecks,
accessibility checks, or other development gates. Those remain in the final
local phase entrypoint.

### 5. Validate the deployed product paths after Pages returns its URL

The deploy job checks the root plus one narrative route and both generated
reference roots using retrying, fail-on-HTTP-error requests. This is validation
of the deployed artifact, not routine development testing.

## Risks / Trade-offs

- **[Risk] Workspace rustdoc is expensive in deployment** → It is a required
  publication artifact and runs only in the deployment workflow; local final
  certification reuses the same deterministic command once.
- **[Risk] Generated tools use absolute links that ignore the project base URL**
  → They are copied to stable subtrees and validated through the actual deployed
  project URL before the deployment job succeeds.
- **[Risk] Removing the SDK workflow removes its independent publication cadence**
  → SDK-path changes trigger the sole portal workflow, preserving freshness
  without competing ownership.
- **[Trade-off] Python remains narrative-only** → Publishing an ungenerated or
  placeholder Python reference would be less truthful than omitting the hosted
  reference until a pinned generator exists.

## Migration Plan

1. Add the fail-closed reference staging script and npm-only Docusaurus build.
2. Rewrite `docs.yml` to use the real generation commands and one final artifact.
3. Delete the competing TypeScript Pages workflow.
4. Run structural controls and the GitHub Actions policy locally; defer the full
   frozen site build and route certification to the final phase change after all
   content is complete.
5. Roll back by reverting this change as one unit. Do not restore only the second
   publisher, because that recreates the observed collision.
