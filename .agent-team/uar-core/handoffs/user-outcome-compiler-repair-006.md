# Bounded compiler repair 006 handoff

Phase: `uar-post-lifecycle-verification-readiness`, `user-outcome-implementation`.
Scope: the three observed P0.7 compiler errors only. Operator verification gate remains **CLOSED**. No build retry is authorized by this repair.

## Source changes

- `src/uar/persistence/scoped.rs:905`: change `begin_knowledge_regeneration`'s target type from the nonexistent `crate::uar::rag::embeddings::EmbeddingFingerprint` to `crate::uar::domain::knowledge::EmbeddingFingerprint`.
- `src/uar/persistence/scoped.rs:918`: make the same type-path correction for `search_knowledge_scoped_in_space`'s space argument.
- `src/llm/liter_driver.rs:727`: replace aggregate `LowerHex` formatting of `Sha256::digest(&wire)` with bytewise lowercase, zero-padded hexadecimal encoding collected into `String`.

The two persistence implementation signatures now name the canonical type already used by the `PersistenceLayer` trait in `src/uar/persistence/mod.rs` and defined in `src/uar/domain/knowledge.rs`. The root cause of the two E0425 errors was an incorrect module path, not an absent domain type.

The digest root cause was E0277: the observed SHA-256 digest array does not implement the aggregate `LowerHex` formatting requested by the prior expression. The replacement uses the existing repository idiom present in `src/llm/team_profile.rs`, `src/uar/persistence/client_tools.rs`, and `src/uar/runtime/observer/channel.rs`. No direct `hex` dependency was found in `Cargo.toml`; no dependency was added. The same `wire` byte slice is hashed with the same SHA-256 function, and each resulting byte emits two lowercase hexadecimal digits. Serialization, byte counts, request counting, algorithm and wire input are unchanged.

## Inspection and limitations

Performed source inspection only: loaded Rust workspace and best-practices skills, read the authoritative waypoint, consulted gotchas, inspected the affected spans, canonical trait/type definitions, manifest and existing digest encoding patterns. Applied exactly the three production-line substitutions above. No new guards, retries, fallbacks, unrelated cleanup or dependency changes were introduced.

No Cargo command, compiler probe, test, lint, formatter, build, review-driven verification or `git diff --check` ran. Compilation and runtime behavior remain unverified. These edits address the recorded diagnostics by source reasoning; they do not constitute a passing build or phase acceptance.

Shared-checkout ownership: only the two assigned source files were edited, preserving existing surrounding work. This handoff is the only additional file authored by this teammate. Source ownership is released to the lead. The lead owns source inventory refresh, amendment/approval packet, canonical ledger and cumulative resource accounting. No unresolved implementation decision remains within this three-line repair.
