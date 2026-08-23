# Validation plan — `uar-documentation-brand-source-review`

1. Compare every copied mark, wordmark, and favicon with its canonical React
   application source.
2. Validate pinned local-search/font configuration and reject hosted search,
   analytics, remote fonts, Ask AI, and stock metadata.
3. Validate UAR tokens, semantic homepage structure, internal routes, visible
   focus, reduced motion, and the absence of stock tutorial assets.
4. Run every negative control and require the current source positive control.
5. Run the Docusaurus TypeScript check, exact dependency resolution, strict
   OpenSpec validation, and the scoped diff audit.
6. Review the sources against the current Web Interface Guidelines while
   retaining browser, accessibility, and production-build work for the final
   certification change.
7. Persist schema-valid evidence and terminate only the bounded source review.
