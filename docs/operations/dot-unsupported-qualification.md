# Disabled capability qualification

`scripts/dot/unsupported_journey.py` is authored and unexecuted. The full-phase
runner calls `run_unsupported(spec, evidence)` after implementation is complete.
Each required storage profile supplies an isolated `unsupported` configuration
with `host`, `provider_port`, `artifact`, and `run_credentials`. Its actual UAR
configuration disables memory and omits supervisor recovery configuration;
its controlled model route and native tools remain available. Credentials include
a real trusted-host principal for the recovery API, plus an ordinary owner.

The driver reads actual operation discovery, attempts HTTP memory controls and
supervisor recovery, and calls the existing native `memory_capture_policy` tool
through a real model run. Discovery must report these capabilities unavailable,
HTTP must return service unavailable, and the native tool must produce an actual
unsuccessful tool result rather than an empty successful response. The model's
final prose is not treated as the tool's outcome.

This proves disabled-operation behavior in each required storage profile. It
does not claim PostgreSQL checkpoint or observer support, nor does it infer
behavior of a nonexistent embedded-runtime interface. The source integration
targets separately exercise explicitly unsupported persistence contracts.
