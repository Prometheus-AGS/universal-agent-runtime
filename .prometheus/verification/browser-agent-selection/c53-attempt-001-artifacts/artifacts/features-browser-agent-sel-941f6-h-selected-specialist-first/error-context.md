# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/browser-agent-selection.feature.spec.js >> The browser shows orchestrator agent selection before the answer >> A writing task and then a math task show each selected specialist first
- Location: target/manual-browser-agent-selection-generated/features/browser-agent-selection.feature.spec.js:6:3

# Error details

```
Error: browserContext.newPage: Executable doesn't exist at /Users/gqadonis/Library/Caches/ms-playwright/ffmpeg-1011/ffmpeg-mac
╔═════════════════════════════════════════════════════════════════╗
║ Video rendering requires ffmpeg binary.                         ║
║ Downloading it will not affect any of the system-wide settings. ║
║ Please run the following command:                               ║
║                                                                 ║
║     npx playwright install ffmpeg                               ║
║                                                                 ║
║ <3 Playwright Team                                              ║
╚═════════════════════════════════════════════════════════════════╝
```