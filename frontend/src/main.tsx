import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import { App } from "./App";
import { DbProvider } from "@/lib/db-context";
import { TooltipProvider } from "@/components/ui/tooltip";
import { bootstrapEntityGraph } from "@/entities/bootstrap";

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
