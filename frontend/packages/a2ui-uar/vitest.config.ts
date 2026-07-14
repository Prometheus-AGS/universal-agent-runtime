import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "happy-dom",
    globals: true,
    setupFiles: ["./test/setup.ts"],
    include: ["test/**/*.test.{ts,tsx}"],
    exclude: ["test/perf/**", "**/node_modules/**", "**/dist/**"],
  },
  resolve: {
    dedupe: ["react", "react-dom"],
  },
});
