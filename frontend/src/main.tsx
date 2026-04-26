import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import { App } from "./App";
import { DbProvider } from "@/lib/db-context";
import { TooltipProvider } from "@/components/ui/tooltip";
import { bootstrapEntityGraph } from "@/entities/bootstrap";
import { initSettingsRealtimeBridge } from "@/stores/settings-store";

// Bootstrap the entity graph engine before rendering
bootstrapEntityGraph().catch((err) =>
  console.error("[entity-graph] bootstrap failed:", err),
);
initSettingsRealtimeBridge();

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
      <TooltipProvider delayDuration={200}>
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
