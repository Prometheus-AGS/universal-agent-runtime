# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/a2ui-browser.feature.spec.js >> Interact with a real A2UI response in the browser >> A rendered calculator sends fresh input back to its agent
- Location: target/manual-a2ui-browser-generated/features/a2ui-browser.feature.spec.js:6:3

# Error details

```
Error: browserContext.newPage: Executable doesn't exist at /Users/gqadonis/Library/Caches/ms-playwright/ffmpeg-1013/ffmpeg-mac
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