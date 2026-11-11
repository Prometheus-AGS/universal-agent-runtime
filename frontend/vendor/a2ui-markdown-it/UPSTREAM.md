# A2UI Markdown adapter

This private workspace package derives from the published Apache-2.0
`@a2ui/markdown-it@0.2.0` distribution. The upstream archive is
[the npm release tarball](https://registry.npmjs.org/@a2ui/markdown-it/-/markdown-it-0.2.0.tgz).
Its SHA-512 integrity is
`sha512-B1nO5Nh2Vw+P0W1DdwYypIayOoAxj3/Fzohxy/q+0TGDG7t4rD+2roUpOnIWW67L7wvGgNclJ8gLzJTkmTcq/A==`.

The `renderMarkdown` implementation, sanitizer, declarations, source maps, and
export path are unchanged. Type-only references to `@a2ui/web_core` resolve
through the paired Zod 4 core. The manifest pins DOMPurify to 3.4.17 and
Markdown-it to 14.3.3. This copy omits upstream test artifacts and `.tsbuildinfo`,
which are outside the package's runtime and type export graph.
