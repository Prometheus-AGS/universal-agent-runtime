## Context

Supply-chain certification requires a successful `.github/workflows/ci.yml` run at the exact candidate SHA. The merged RC3 source fails that gate because the general Clippy job invokes Clippy across path dependencies, including a vendored crate that denies its own warning set. The release workflow already uses the authoritative locked `server-full` no-dependency Clippy command successfully.

## Goals / Non-Goals

**Goals:**

- Make the exact-SHA CI lint gate exercise the supported `server-full` product surface.
- Keep dependency-owned Clippy warnings outside the UAR lint policy boundary.
- Statically enforce the authoritative command used by candidate certification.

**Non-Goals:**

- Change vendored dependency source or lint policy.
- Change runtime, frontend, provider, or realtime behavior.
- Treat Experimental Windows failures as release blockers.

## Decisions

- Use `cargo clippy --locked --no-default-features --lib --features server-full --no-deps`, matching the authoritative release workflow. This is preferred over patching 138 dependency warnings because those warnings belong to the excluded vendored crate and are unrelated to the supported UAR source surface.
- Align the adjacent Cargo check with the locked `server-full` checkpoint so CI and release certification use the same product contract.
- Extend the existing supply-chain static validator to assert both CI commands because supply-chain evidence consumes the exact-SHA CI result.
- Use the deterministic BDD fixture model in release-job environment defaults; archive smoke still receives a configured non-secret provider.
- Install the same Linux build prerequisites in resilience as other server-full jobs, including `protobuf-compiler` for `build.rs`.
- Give Docker 45 seconds to observe the runtime's 30-second graceful-shutdown contract before escalating to SIGKILL.

## Risks / Trade-offs

- **Risk:** Removing the old mixed feature set from the primary lint job could reduce coverage. **Mitigation:** the CI release-bundle matrix continues to check the supported bundles independently, while the release gate is explicitly scoped to `server-full`.
- **Risk:** Any source correction invalidates RC3 evidence. **Mitigation:** merge this isolated fix and create a newly signed RC4; never move RC3.
- **Risk:** Candidate failures discovered after an immutable tag require another candidate. **Mitigation:** mine every completed failed job before creating the next signed tag and supersede tags rather than moving them.
