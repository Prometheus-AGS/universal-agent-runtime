import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import { compileA2uiTestDecorators } from "../a2ui-core/vitest-decorators.ts";

export default defineConfig({
  plugins: [react(), compileA2uiTestDecorators()],
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
