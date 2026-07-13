# Decision Log — uar-final-production-hardening-2026-07

## Durable product decisions

- React 19 + TypeScript is the authoritative first-party UI.
- `server-full` is the customer/BossFang sidecar product.
- BossFang consumes UAR as a supervised sidecar; embedded/library consumption is not required for this release.
- Linux and macOS are Stable; Windows is Experimental and nonblocking.
- The default UAR port is 1906 and remains configurable.

## 2026-07-13 — Operator execution lock

The objective is 24/24 production completion, not workflow-green optimization. Batch implementation first, use `cargo check` at cohesive implementation checkpoints, validate the completed product once, and perform one immutable certification/release sequence. CI is asynchronous evidence. Do not poll it while actionable implementation or release work remains. Operator directions take precedence over stale KBD history and agent verification preferences.

Consequences:

- Active KBD files contain only changes 20–24 and current completion state.
- Historical assessments and sequential plans are retained only in Git history.
- Requirements are classified as implementation, evidence, time-bound, or operator-authorized.
- No supported-source change is made solely to chase Experimental Windows or cosmetic workflow status.
