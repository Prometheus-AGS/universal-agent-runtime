# UAR documentation site

> **Current authority:** [UAR documentation portal](/docs/intro). This directory
> owns the Docusaurus source and its deployment-only GitHub Pages workflow.

The site uses the package-local npm lockfile and the UAR brand assets under
`static/img/brand/`. Product pages are authored in `docs/`; shared navigation
and deployment configuration are in `sidebars.ts` and `docusaurus.config.ts`.

## Local development

From the repository root:

```bash
npm --prefix website ci
npm --prefix website run start
```

The final documentation certification change owns the production build,
representative browser routes, responsive light/dark screenshots, keyboard
behavior, and accessibility checks. Do not treat the development server as
publication evidence.

## Local source checks

After documentation content is complete:

```bash
npm --prefix website run typecheck
npm --prefix website run lint
npm run docs:publication:validate
```

## Publication

`.github/workflows/docs.yml` is the single GitHub Pages publisher. It may build
and validate the deployable Pages artifact because those checks are part of the
deployment itself. It must not run product unit, integration, conformance,
linting, typecheck, or other routine development suites. Local certification
must pass before the documentation commit is pushed.
