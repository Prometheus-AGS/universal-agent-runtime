# FC4 handoff — reduced runner and release contract

Date: 2026-10-17. Child Execute task `3.1` / FC4. Verification gate:
**CLOSED**. This is source implementation and documentation, not acceptance.

## Delivered files and interfaces

FC2 and FC3 returned their assigned adapter files with source-only handoffs in
`handoffs/FC2.md` and `handoffs/FC3.md`. The lead integrated those interfaces
in these owned files:

- `scripts/release/uar_1_1_1_acceptance.py`: local six-group and published
  `streams,recovery` programs; stage-specific one-failed-group repair selection;
  selected/omitted program fields in the receipt; closed `prebuilt` image mode.
  The existing four native source supplements still run only with the local
  groups. The functional invocation inspects prepared image bytes and does not
  build or pull in prebuilt mode.
- `scripts/release/uar_1_1_1_manifest.schema.json` and
  `scripts/release/uar_1_1_1_manifest.example.json`: prebuilt image content ID,
  hashed preparation receipt and stage-specific group selection. The example
  retains placeholders and is not executable.
- `docs/operations/uar-1.1.1-acceptance.md`, `docs/releases/v1.1.1.md`, the
  existing OpenSpec proposal/design/tasks/verification and capability deltas,
  parent plan/ownership/spec fragments, and append-only
  `.prometheus/decisions.md`: narrowed assurance, its functional controls,
  prebuilt image identity, fresh supervisor preparation and lost per-child
  authority observations are explicit.
- Child `scenario-disposition.md`: all 80 historical labels are routed to 65
  final local labels. Sixteen are folded or replaced, one user-facing
  prompt/skill/timing label is added. The actual execution reduction also
  removes partition and repeat cycles and changes published group executions
  from eighteen to four.
- Child `supervisor-preparation.md`: source-backed local and published image,
  installation, qualified-controller, real staged work, physical guest boot,
  replacement and proof-binding procedure. It names old private helpers as
  historical and leaves all future identities unfilled.

## Current source and unresolved inputs

The shared checkout contains FC2, FC3 and FC4 edits. No frozen current
source/fixture digest, new image ID, or qualified replacement receipt exists.
The existing private rc05/rc07 install helpers bind old identities; they
cannot install or qualify this candidate unchanged. FC5 must make a new
whole-parent-and-child inventory and exact source identity, then present the
operator with the distinct build, installation, staging, physical-boot, review
and functional commands as the required inputs become concrete. FC6 must
render the parameterized private guest commands with the actual image and
qualified state before any use. A missing qualified controller or fresh
same-image staged predecessor blocks certification.

The current reduction certifies explicit user-selected skill attachments,
not model-selected skill quality. The controlled provider yields diagnostic
timing, not paid-provider latency or an independently measured speedup. The
published program gives up direct per-child budget, compiler, credential and
MCP failure observations; those remain local release blockers. No outcome is
claimed for the 65 labels or either architecture.

## Source-only process and verification status

The runner and documentation were edited in disjoint files while FC2/FC3
were finishing their assigned adapters, before their final handoffs arrived.
No shared file was concurrently written; FC4 consumed both completed
handoffs before this handoff. This sequencing differed from the plan's strict
dependency order and is recorded here rather than hidden.

Source inspection used `rg`, `sed`, `cat` and `git diff` on named files. Edits
used `apply_patch`. No test, lint, format/type check, compiler/import probe,
Docker command, build, acceptance script, external gate or `git diff --check`
ran. The operator verification lock remains CLOSED. Syntax, schema/runtime
compatibility, behavior, image identity, supervisor qualification and timing
are all unverified until the whole-phase boundary and exact operator
instructions. No runtime product behavior or unrelated guard was added.
