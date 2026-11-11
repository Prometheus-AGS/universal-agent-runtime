# A2UI React v0.9 compatibility renderer

This private workspace package derives from the published Apache-2.0
`@a2ui/react@0.12.0` distribution. The upstream archive is
[the npm release tarball](https://registry.npmjs.org/@a2ui/react/-/react-0.12.0.tgz).
Its SHA-512 integrity is
`sha512-zGI6kNjus8m0OM4V/A0S3g3YDFodtd82C+f91CECmt9WQniJtFvlSLZqOu6xBz2R3uUh7aviOOZUhTCVft85Ng==`.

This package retains the v0.9 React `basicCatalog`, its 18 built-in controls,
and its renderer exports for compatibility consumers. The runtime and
TypeScript declarations are the original artifacts. Catalog schema creation
and binding resolve through the paired Zod 4 web core under the original
`@a2ui/web_core` import specifier. The React artifacts do not import `zod` or
`@a2ui/markdown-it`. Applications configure their Markdown renderer through the
existing context. The v0.8 renderer's Markdown parser is pinned to 14.3.3.
