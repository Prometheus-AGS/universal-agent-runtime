# Surrealism verifier RustCrypto dependencies

The source and tests are from the published `surrealism-verify` 0.7.0
crate at SurrealDB commit `284fe93afc851c3baec2c262c01289602021a70d`.
The original crate SHA-256 is
`bc68e15ea6a12eed7ffb61e16c6adb19ed6657fbba837ba098b04f5ef0b0d66c`.

This overlay changes dependency versions only. The previous P-256,
P-384 and OpenSSH verification dependencies selected `crypto-bigint`
0.5.5. RustSec lists 0.7.6 as the first fixed version. P-256 and P-384
0.14 and RustCrypto `ssh-key` 0.7.0-rc.12 use the updated curve stack.
`signature` 3 matches the OpenSSH verifier trait and
`ed25519-dalek` 3 matches its Ed25519 dependency. The official
`ssh-key` crate SHA-256 is
`0701b552e6bd8f13a6e07bde25307c32d7daad80a79b65832d110c0416e7b638`.

The OpenSSH dependency still enables only its existing `alloc`,
`ed25519` and `p256` features. Existing code checks the namespace, hash
algorithm, recorded key and key type before verifying. The Sigstore
bundle, certificate, transparency log, timestamp and trust-root checks
retain the published implementation. Upstream conformance fixtures,
OpenSSH interoperability tests and the native differential verifier
are retained.

- [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html)
- [Published verifier](https://crates.io/crates/surrealism-verify/0.7.0)
- [Published OpenSSH library](https://crates.io/crates/ssh-key/0.7.0-rc.12)
