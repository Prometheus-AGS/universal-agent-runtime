import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import { App } from "./App";
import { DbProvider } from "@/lib/db-context";
import { TooltipProvider } from "@/components/ui/tooltip";

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
