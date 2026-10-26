
## 2026-10-26 — VERIFIED: streamed default-agent answer on Mac

Given actual native UAR1.3.0 and configured real Liter/MiniMax-M3, when the default agent was asked to count1–20 with streaming, then it emitted 3 content events before completion, assembled the correct sequence, and ended with finish_reason=stop and [DONE].

Cucumber:1scenario passed,6steps passed, exit0; request/stream duration2552ms. [Report](2026-10-26T06-56-36.601Z-streaming-macos.json), [verification summary](2026-10-26T06-56-36.601Z-streaming-macos.verified.json), [actual runtime/build identity](2026-10-26T06-56-36.601Z-streaming-macos.runtime.json).

The plain-prompt behavior was not rerun. The earlier empty discovery and failed sideband-event parsing attempts are retained; neither counted as verified. Final parser distinguishes documented runtime.* sideband events without weakening content/terminal assertions. Stopped after this pass; no other behavior started.
