# Object compressed-section decoder backport

Base crates.io object 0.39.1, SHA-256
2e5a6c098c7a3b6547378093f5cc30bc54fd361ce711e05293a5cc589562739b.
Use the published object 0.40.0 Zstandard decode branch and the fixed
ruzstd 0.9.1. Preserve the existing public Object API, Zlib path and final
uncompressed-length check required by the pinned Wasmer compiler.

- https://docs.rs/crate/object/0.40.0/source/src/read/mod.rs
- https://rustsec.org/advisories/RUSTSEC-2026-0344.html
