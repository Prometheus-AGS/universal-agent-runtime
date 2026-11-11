# A2UI React reference renderer

This private workspace package derives from the published Apache-2.0
`@a2ui/react@0.14.0` distribution. The upstream archive is
[the npm release tarball](https://registry.npmjs.org/@a2ui/react/-/react-0.14.0.tgz).
Its SHA-512 integrity is
`sha512-kew3gVx4/U4qZ41FwyanDndCuBsx3s+5kPzjBt5N9w8E6UKSfwJpOmWD3Vy5WN5bxuiQKmorN+Ozh59ZLrwkMA==`.

The package uses the paired Zod 4 web core under the original `@a2ui/web_core`
import specifier. The React runtime passes catalog schemas to that core's
`GenericBinder`. The React declarations use Zod 4's public `ZodType` constraint,
and the package requires Zod 4.6.7 as a peer. The Markdown parser is pinned to
14.3.3. Upstream runtime bundles, source maps and export paths are retained.

The manifest drops the unused `@a2ui/markdown-it` dependency. Applications
configure their Markdown renderer through the existing context.
