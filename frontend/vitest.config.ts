import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "node:path";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "happy-dom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
    exclude: ["**/node_modules/**", "**/packages/**", "**/dist/**", "e2e/**"],
  },
  resolve: {
    dedupe: ["react", "react-dom", "use-sync-external-store", "zustand"],
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
