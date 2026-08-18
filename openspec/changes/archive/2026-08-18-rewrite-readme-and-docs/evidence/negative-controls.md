# Negative controls — `rewrite-readme-and-docs`

Date: 2026-08-18

## Broken-link build guard

A temporary, normally routed page was added at
`website/docs/broken-link-negative-control.md` with this link:

```markdown
[This target must not exist](./missing-customer-documentation-target)
```

Command:

```bash
cd website
npm run build
```

Observed output and exit:

```text
[ERROR] Error: Unable to build website for locale en.
[cause]: Error: Docusaurus found broken links!
- Broken link on source page path = /universal-agent-runtime/docs/broken-link-negative-control:
   -> linking to ./missing-customer-documentation-target
[exit 1]
```

The temporary page was deleted. The same production build then exited 0 and
generated the static site.

An earlier underscore-prefixed control was ignored by Docusaurus as a partial
and incorrectly exited 0. It is retained here because it demonstrates why the
observed failing control, not merely a configured assertion, is required.
