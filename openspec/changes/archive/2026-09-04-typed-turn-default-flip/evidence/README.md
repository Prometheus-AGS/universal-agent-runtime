# Live shadow evidence — 2026-09-04

Phase-end command (exit 0):

```sh
UAR_SMOKE_MODEL=k3 UAR_SMOKE_LOG=info node openspec/changes/typed-turn-default-flip/evidence/live-shadow.mjs target/debug/uar-sidecar
```

Observed output:

```text
sidecar ready; isolated database; harness=shadow
running basic-user-turn
basic-user-turn: completed; 1 comparisons; zero unexpected differences
running host-instructions
host-instructions: completed; 1 comparisons; zero unexpected differences
```

The JSON output is transcribed without field changes in `live-shadow-report.json`.
Both runs emitted real provider text, completed, dispatched the legacy path,
and emitted one nonempty shadow report with no differences or exemptions.
Scratch receipt: `/var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/uar-live-shadow-KuhlKb`.
The sidecar was built from this working tree with the legacy default before
the flip. Authentication and shipped Cedar policies stayed enabled; the runner
explicitly selected no skills, tools, MCP servers, knowledge bases, or memory.

The uncomfortable thing: this is two short cases on one model, not a broad
provider, memory, tool, MCP, or child-thread parity matrix. The separate checked-in
corpus has three cases. Neither evidence set proves every production combination.
Cancellation and provider errors are failures, never successful parity evidence.

An artifact-only critic found no concrete false-positive gate, and identified a
credential-cleanup ordering issue. The runner now redacts scratch configuration
before writing retained logs; that cleanup-only adjustment followed the passing run.
`node --check` and `git diff --check` exited 0 after the adjustment.
An initial launch failed because ignored stdin caused sidecar EOF shutdown;
the passing invocation kept stdin open. No provider request ran in that attempt.
