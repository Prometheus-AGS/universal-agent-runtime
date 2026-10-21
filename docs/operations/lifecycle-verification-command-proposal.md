# Lifecycle command proposal — verification CLOSED

The verification and native-preparation commands below have not run. The final section separately records the approved Docker delivery command, which completed successfully at `32cb3bcfc9691212ec0b0359ce22474968bc6850` for both Linux architectures. It published `sha256:8e6fb6f453323fa921d956adb78d15b4ab4ae6137acb3cc5827a83bcd9bc1fc8`. That narrow delivery exception does not authorize any command below. Run only commands the operator explicitly approves, serially, after reconciliation of the full phase and exact prerequisites. Approval for one command grants no other command or hidden preflight.

## Local embedding assets and actual inference

Working directory: `/Volumes/T7/Projects/prometheus/universal-agent-runtime`.
The preparation command downloads only the five public, immutable MiniLM
assets declared in the fixture lock. It checks byte lengths and published
Git-blob/SHA identities, requires different primary weights, and writes a new
external directory. It makes no hosted inference request and reads no personal
configuration. It is a separately proposed preparation/validation command,
not work already executed under the closed gate.

```sh
python3 tests/fixtures/local_embedding_lifecycle/prepare_models.py --primary-dir /Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models --incompatible-dir /Volumes/T7/uar-local-embedding-minilm-1110a243fdf4
```

The real-model target uses the repository's primary assets and a second local
engine over compatible assets. The test copies the primary into its owned
temporary fixture directory. It requires actual incompatible-model inference;
missing assets cannot become a passing skip.

```sh
UAR_EMBEDDING_LIFECYCLE_PRIMARY_DIR=/Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models UAR_EMBEDDING_LIFECYCLE_BACKUP_DIR=/Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIR=/Volumes/T7/uar-local-embedding-minilm-1110a243fdf4 UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_MODEL=sentence-transformers/all-MiniLM-L6-v2 UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIMENSION=384 cargo test --locked --no-default-features --features server-full --test embedding_lifecycle_failover -- --test-threads=1
```

This target covers live generations 1→2→3→4→5, durable ingestion and query
continuity, equal-dimension incompatible-model refusal, state preservation and
parent-owned cleanup. It does not by itself certify process-restart settings
hydration, authenticated settings mutation, the separate MemoryService
embedding provider, or the entire frozen child contract. Keep those evidence
requirements intact.

## Resource ownership and sidecar integration

Same UAR working directory. These are separate proposed production-path
integration targets; neither substitutes for complete release acceptance.

```sh
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend,test-probes --test lifecycle_owned_resources -- --test-threads=1
```

```sh
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend,test-probes --test sidecar_launch_security -- --test-threads=1
```

Retain actual cleanup reports, child exits and per-case sanitized failure
receipts. No additional polling, lock-probe loop or elapsed-time success test
is implied. Native macOS execution does not certify Windows overlapped pipes,
WASM, every remote transport or a physical engine-close fault.

## Fresh Boss payload and local customer package

Prerequisite: an actual UAR source commit is published and the Boss schema-2
pin names that commit. The existing old pin is not sufficient. This source prerequisite now exists: Boss pins `32cb3bcfc9691212ec0b0359ce22474968bc6850`.
The producer refuses a selected commit whose producer bytes do not match.
No candidate artifact exists merely because its source pin is now recorded.

From `/Volumes/T7/Projects/prometheus/universal-agent-runtime`, on native Apple
Silicon, propose this exact build command, using the actual pinned source:

```sh
node scripts/build-boss-candidate.mjs 32cb3bcfc9691212ec0b0359ce22474968bc6850 /Volumes/T7/uar-boss-lifecycle-candidate-20261021-01
```

The output directory must be new. The command exports committed Git objects
and builds with a fresh target directory. It invokes the exact Cargo build
documented in `lifecycle-candidate-payload.md`; that nested build is part of
this proposed command, not a separately granted preflight.

From `/Volumes/T7/Projects/prometheus/the-boss`, propose:

```sh
THE_BOSS_LOCAL_UAR_SOURCE_DIR=/Volumes/T7/Projects/prometheus/universal-agent-runtime THE_BOSS_LOCAL_UAR_CANDIDATE_RECORD=/Volumes/T7/uar-boss-lifecycle-candidate-20261021-01/uar-sidecar-darwin-arm64.json pnpm build:mac:arm64:local
```

This validates and packages the explicit local candidate. The required public
customer build remains a separate gate:

```sh
pnpm build:mac:arm64
```

That public command forces public payload selection and cannot certify this
new source until a public payload is actually bound to it. Do not substitute a
downloaded old sidecar, fabricate an artifact receipt, or relabel other
platforms. Windows needs its own exact command packet and real execution.

## Full local release acceptance

From the UAR working directory, the planned runner command remains:

```sh
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Volumes/T7/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/function-first-local.json
```

The manifest is not ready for these source bytes. Fresh actual image identity,
qualified same-image supervisor/guest preparation and fixture inventory must
be bound through separately authorized exact preparation commands first.
Independent completed-artifact review is also pending. No guessed preflight,
old receipt, unit-only pass or source inventory satisfies those prerequisites.
The historical 503 body and individual sidecar traces remain missing.

## Source publication and image delivery

Source publication was requested by the operator. The operator explicitly approved source-only hook exceptions and the
requested deployment build. Scoped source commits and draft PRs are published;
`lifecycle-publication-approval.md` records the exact narrow authorization.
The first two invocations failed on both architectures; their bounded source repairs and renewed Boss pins are recorded. The third invocation completed success: https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39532243053, immutable source `32cb3bcfc9691212ec0b0359ce22474968bc6850`. Both Linux images and the combined index were published. Routine verification remains CLOSED:

```sh
/opt/homebrew/bin/gh workflow run deploy.yml --repo Prometheus-AGS/universal-agent-runtime --ref codex/uar-1-1-1-implementation
```

This is the requested Docker delivery path, not general verification. The actual combined digest is `sha256:8e6fb6f453323fa921d956adb78d15b4ab4ae6137acb3cc5827a83bcd9bc1fc8`, recorded from the existing workflow inspection. Its published tag is `sha-32cb3bcfc9691212ec0b0359ce22474968bc6850`. Dispatch alone was never used as publication evidence. No local
Docker build or routine GitHub Actions test is included.
