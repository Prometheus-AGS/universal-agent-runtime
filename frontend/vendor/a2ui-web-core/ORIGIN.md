# A2UI Web Core Zod 4 candidate

This local package carries the production TypeScript and schema resources from
the Apache-2.0 licensed upstream `@a2ui/web_core@0.14.0` package. Its source maps
embed the TypeScript sources. All 225 embedded sources matched the upstream
[release commit 45633b97](https://github.com/a2ui-project/a2ui/commit/45633b97636daf89bafb43fa7db23e3cd916461d)
byte for byte before editing. The [registry artifact](https://registry.npmjs.org/@a2ui/web_core/-/web_core-0.14.0.tgz)
has SRI `sha512-OEJOgcfRMW0lwActVl1NKToxE2Ff5G3mIBHDB6gKm0elGZSwAhB7bj+Ef2a/h2nrL+57VPnSR67YSNz9JVV4yw==`.
The original license, readme, changelog and source headers are retained.

The candidate keeps the upstream export subpaths and runs with native Zod
4.6.7. Its browser-facing entry points configure the non-JIT parser for strict
content security policies. Catalog loader bounds use Zod's early array-size
check. Catalog schema export uses the native JSON Schema emitter, with A2UI
reference and closure rules retained by the catalog shaper. The binder,
reference-field analysis and known function-argument handling traverse native
Zod 4 shapes. Zod's safe-integer range applies at parse time. Catalog output
continues to include only explicitly authored integer bounds. Lit uses standard
decorator accessors so the workspace can compile the source with its existing
decorator settings.

The workspace consumes TypeScript directly. For an independent consumer,
run the package's `build` script and stage the compiled outputs with
`stage:package <new-directory>`. The staging script writes a normal npm package
manifest with JavaScript and declaration entry points. The package's
`verify:migration` script exercises the compiled catalog loader, binder and
protocol schemas against the retained wire fixtures.

The package name `@prometheus-ags/a2ui-web-core-zod4` identifies a UAR candidate.
Publication has a separate authorization and release workflow.
