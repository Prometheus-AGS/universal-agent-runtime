# Findings: direct-network-facing-vulns

## hickory-proto — dead dependency, not reachable

`cargo tree --target all --all-features -i hickory-proto` shows:

```
hickory-proto v0.25.2 default,futures-io,std,tokio
├── hickory-resolver v0.25.2 default,system-config,tokio
│   └── microsandbox-network v0.3.14
│       ├── microsandbox v0.3.14 (behind UAR's optional `sandbox-microsandbox` feature)
│       └── microsandbox-runtime v0.3.14
└── microsandbox-network v0.3.14 (*)
```

Note the activated feature set for `hickory-proto` is
`default,futures-io,std,tokio` — **no `dnssec-ring` or `dnssec-aws-lc-rs`**,
which `RUSTSEC-2026-0118`'s vulnerable `DnssecDnsHandle` code requires to
even be compiled in.

Grepped every `microsandbox*` crate's cached source
(`~/.cargo/registry/src/.../{microsandbox,microsandbox-network,microsandbox-runtime,...}-0.3.14`)
for `hickory`, `Resolver`, `lookup_ip`, `resolve` — **zero matches**. The
dependency is declared in `microsandbox-network`'s `Cargo.toml` but never
actually invoked anywhere in its own code or any sibling microsandbox
crate. This is a dead transitive dependency in the published crate we
consume.

| Advisory | Status | Note |
|---|---|---|
| `RUSTSEC-2025-0006` (DNSSEC RRSIG bypass) | Already patched | Fix range `>= 0.25.0-alpha.5`; our locked `0.25.2` already satisfies it. Doesn't appear in `cargo audit` output. |
| `RUSTSEC-2026-0118` (NSEC3 unbounded loop) | Not reachable | Requires `dnssec-ring`/`dnssec-aws-lc-rs` feature (not activated) AND actual `DnssecDnsHandle` usage (none found anywhere in the microsandbox family). `patched = []` within the 0.25.x line regardless — only fix is `hickory-net >= 0.26.1`, and `microsandbox-network`'s manifest pins `^0.25` so this isn't available to us without their upstream bumping first. |
| `RUSTSEC-2026-0119` (O(n²) name compression, message encoding) | Not reachable | Same dead-dependency situation — no code path encodes a hickory-proto message at all, let alone one with attacker-influenced record counts. Fix requires `>= 0.26.1`, same manifest-pin blocker as above. |

**Disposition: not reachable, no action taken.** If UAR ever adds actual
DNS-resolution logic that calls into `microsandbox-network`'s (currently
unused) hickory dependency, this disposition should be re-checked.

## tokio-tar — eliminated by removing the unused `testcontainers` dev-dependency

`cargo tree --target all --all-features -i testcontainers` showed
`tokio-tar` reachable only via `testcontainers` (a dev-dependency). A
full-repo grep for `testcontainers::`, `GenericImage`, `ContainerAsync`,
and `testcontainers::runners` found **zero actual usage** — the only hit
for the bare string `testcontainers` was a code comment in
`src/uar/security/credentials/store.rs:530` ("This repo has no existing
testcontainers wiring for Postgres...").

Removed `testcontainers = "0.23"` from `Cargo.toml`. This is a complete
fix, not a mitigation: `tokio-tar` (and 9 other crates exclusively pulled
in for it — `bollard`, `bollard-stubs`, `hyperlocal`, `hyper-named-pipe`,
`parse-display`, `parse-display-derive`, `redox_syscall`, `structmeta`,
`structmeta-derive`) are gone from `Cargo.lock` entirely, confirmed by
`cargo audit` no longer listing `RUSTSEC-2025-0111`.

### A note on the fix process

An initial attempt to regenerate `Cargo.lock` via a bare `cargo update`
(no `-p` scope) bumped ~190 unrelated packages across the whole graph —
reverted immediately (`git checkout -- Cargo.lock`) since that's far
outside this change's scope. The actual fix was: edit `Cargo.toml` only,
then let `cargo check` produce a scoped re-resolution — which, once the
registry index wasn't polluted by the earlier broad `cargo update`,
produced exactly the minimal diff described above (183 deletions, 0
additions).
