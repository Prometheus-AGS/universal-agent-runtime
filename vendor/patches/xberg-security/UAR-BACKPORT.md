# Xberg Calamine security backport

This production source export is Xberg 1.4.0 from the exact [Know-Me-Tools/Xberg revision](https://github.com/Know-Me-Tools/xberg/commit/ffb3b1a25b6c14c13858142997dd82a16d5b2f35) used for RC4. The Calamine requirement follows [Xberg #2416](https://github.com/xberg-io/xberg/pull/2416), merged at `803749692abb0e37a0f3f3afa379cffcb114199e`, and selects Calamine 0.38.0 or a compatible patch release for [RUSTSEC-2026-0436](https://rustsec.org/advisories/RUSTSEC-2026-0436). The source export keeps the original extractor implementation while selecting the patched CFB/OLE2 parser.

The exported files are `crates/xberg/{Cargo.toml,README.md,build.rs,src,stopwords}` and the repository's `LICENSE`. The standalone manifest expands the original workspace package metadata, dependency defaults and lints. It retains the 14 features selected by UAR and their original feature definitions, and all 81 Xberg dependency names in UAR's pre-backport lockfile. Each retained dependency keeps its target conditions and options. The Calamine version is the only changed dependency range.

Cargo's local-path resolution also evaluates optional dependencies that UAR never selects. The export excludes upstream-only features and dependency branches, such as Xberg's separate ONNX Runtime backend, along with upstream tests, benchmarks, and development dependencies. These upstream feature names remain declared to the compiler's cfg checker, and their code remains in the source export. The OCR crate retains its exact Git revision, including the pinned OCR data and its license.

The build ID incorporates the source revision, the Calamine backport, the
packaged source and workspace lockfile content. This distinguishes committed
and local extractor changes in Docker contexts and Git checkouts. An explicit
`XBERG_BUILD_ID` retains its original override behavior.

Remove this export when UAR adopts a verified Xberg successor containing the fix.
