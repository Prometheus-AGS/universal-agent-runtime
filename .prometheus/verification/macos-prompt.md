
## 2026-10-26 — VERIFIED: default-agent prompting on Mac

- Given: native UAR1.3.0 running locally with the configured Liter/MiniMax-M3 model, independently accepted by UAR's provider-verification operation (HTTP200, received_text=true).
- When: `node scripts/behaviors/prompt.mjs` sent default-agent the question “What is35+35?”
- Then: HTTP200, answer `70`, model `liter/MiniMax-M3`,1060ms, process exit0.
- [Actual result](2026-10-26T06-38-40.440Z-prompt-macos.json). [Runtime identity and setup evidence](2026-10-26T06-38-40.440Z-prompt-macos-runtime.json).
- Artifact SHA256: `25151ae66312e41c99060971645b2414b28817669a164f064cd59d3b0576828c`; built source `sha256:28137dc46cbb05aa9169f93b46232ea6aafc8d3f04e7c841274e4bbf4d109018`. This is the existing native build, not a rebuilt certification of all current source edits.
- Earlier422 failures remain recorded. GPT-5.4's configured subscription token was expired; MiniMax-M3 succeeded.
- Scope: one prompting behavior on this Mac/runtime; no other workflow or platform is verified.
