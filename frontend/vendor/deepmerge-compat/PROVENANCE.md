# Deepmerge compatibility for the Svelte Vite plugin

The starting point is `dist/cjs.js` from the published `deepmerge@4.3.1` archive with integrity `sha512-3sUqbMEc77XqpdNO7FRyRog+eW3ph+GYCbj+rK+uYyRMuwsVy0rMiVtPn+QJlKFvWP/1PYpapqYn0Me2knFn+A==`. The upstream MIT license is included verbatim.

The Svelte plugin imports the callable CommonJS default and uses its custom array replacement callback when merging local configuration. This package retains that behavior. It discards `__proto__` from either input, defines own data properties on the destination, and aborts if an input refuses a property lookup. These changes address the destination prototype mutation described in [GHSA-gcqx-w48m-w2h4](https://github.com/advisories/GHSA-gcqx-w48m-w2h4) and its [upstream report](https://github.com/TehShrike/deepmerge/issues/273).

The two workspace overrides select this local package only for the pinned Svelte Vite plugin. The package version identifies a private compatibility package. It does not claim an upstream patched release.
