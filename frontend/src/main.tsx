import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
// Brand fonts are bundled, not linked from a CDN, so the app renders correctly
// offline, under a strict CSP and inside the Tauri desktop shell.
import "@fontsource-variable/inter";
import "@fontsource-variable/space-grotesk";
import "@fontsource-variable/jetbrains-mono";
import "./index.css";
import { App } from "./App";
import { DbProvider } from "@/lib/db-context";
import { TooltipProvider } from "@/components/ui/tooltip";
import { bootstrapEntityGraph } from "@/entities/bootstrap";
import { bootstrapTheme } from "@/lib/theme-boot";

// Apply the user's stored theme (uar-theme localStorage) before first paint.
// Without this, index.html's class="dark" always wins and light theme is
// unreachable (found by the uar-ui-verify-gates screenshot matrix).
bootstrapTheme();

// Bootstrap the entity graph engine before rendering. The settings
// change-bus is now driven directly by the SSE adapter in
// `entities/sync.ts` — no separate bridge initialization is needed.
bootstrapEntityGraph().catch((err) =>
  console.error("[entity-graph] bootstrap failed:", err),
);

if (import.meta.env.DEV) {
  void import("@/entities/runtime-replay-test-helper").then(({ installRuntimeReplayTestHelper }) => {
    installRuntimeReplayTestHelper();
  });
}

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("Root element not found");

createRoot(rootElement).render(
  <StrictMode>
    <DbProvider>
      <TooltipProvider delay={200}>
        <App />
      </TooltipProvider>
    </DbProvider>
  </StrictMode>,
);

if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch(() => {});
  });
}
