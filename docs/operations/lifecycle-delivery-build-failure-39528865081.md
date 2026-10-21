# Replacement lifecycle Docker delivery failure

Run: https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39528865081

Immutable source: `6d50cb8a05fb2af33e0928dc8d7ac3ff0c5f3e46`.
ARM64 job: `118989856797`. AMD64 job: `118989856885`.

The authorized ARM64 image build failed in the production Cargo build with
exit101. The compiler reported two errors with one source cause:

```text
error: future cannot be sent between threads safely
  --> src/server.rs:319:17
  let owner = tokio::spawn(async move {

note: future is not Send as this value is used across an await
  --> src/server.rs:1797:54
  policy_count = engine.policy_count().await,
```

The two diagnostics identify `std::fmt::Arguments` and `&dyn tracing::Value`
retained across the same await inside `info!`. Startup now runs in the owned
server task, so those tracing temporaries cannot cross a suspension point.
Resolve the awaited count first and log the resulting value. Preserve the
owned task, actual join, startup checkpoints and governance authority.

Task9.5 covers this repair and the same inline-await startup logging pattern
in `src/server.rs`. Task8.5 requires the resulting actual source commit to be
pinned by Boss before renewed inventory and another delivery invocation.
Warnings are outside this bounded repair. No local compiler probe or other
verification is authorized.

At this receipt boundary AMD64 is still active. ARM64 exported no image digest,
so no multi-architecture image can be published from this run. The parent run
and AMD64 terminal outcomes must be recorded when observed; do not infer them.

Verification remains **CLOSED**. This failed deployment build is neither
acceptance nor evidence that the surrounding lifecycle behavior works.


## Terminal disposition

The run completed with failure. AMD64 job118989856885 failed after43m20s and its actual log reported the same two non-Send diagnostics at server.rs1797 as ARM64. ARM64 failed after29m45s. Manifest job119002931249 was skipped. Neither architecture exported a digest; no image was published. No cancellation occurred. Source32cb3bcf and matching Boss pin already contain the bounded startup repair. Current authorized run39532243053 remains active. These durations describe observations only; they are not correctness deadlines.
