# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/browser-history-reload.feature.spec.js >> A browser conversation reloads from saved history >> Reopen a saved conversation after reload and continue from its earlier fact
- Location: target/manual-browser-history-reload-generated/features/browser-history-reload.feature.spec.js:6:3

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