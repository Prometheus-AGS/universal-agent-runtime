# PGlite Oxide 0.5.1 Intel macOS overlay

This copy is based on the published `pglite-oxide` 0.5.1 crate. Its
original package SHA-256 is
`724b405e433008d85390c80e57b11c2d7d5e3c8a7ec0b1a29e27a2a7ab55c33f`.
The Intel spike's workspace lock resolves this overlay and its dependencies.

On Intel macOS with bundled assets, the overlay uses Wasmer LLVM to compile
WASIX modules at startup. Each module must match the SHA-256 in the bundled
`pglite-oxide-assets` 0.5.1 manifest before it is compiled or cached.
The published runtime archive SHA-256 is
`86290d461e59cca2c4a8024b6d9ce6792f29d737d233a301552a4cf944958b70`.
The PostgreSQL runtime module SHA-256 is
`fc9098cd176d162380c149e26d07dd8f06f67f59c82eb3bb5bc3a34a2cd596a9`.

Wasmer's compiler pin requires LLVM 22.1. Set `LLVM_SYS_221_PREFIX` to the
installed LLVM 22.1 prefix when running the locked Intel spike.
