import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

/**
 * Performance-measurement harness config (see test/perf/README.md).
 *
 * This runs the same suite as vitest.config.ts but scoped to
 * `test/perf/**`, and is intended to be invoked as its own CI step
 * (`pnpm --filter @prometheus-ags/a2ui-uar run perf`) so a future budget
 * gate can fail independently of functional test failures without being
 * bundled into the same job/report as `pnpm test`.
 */
export default defineConfig({
  plugins: [react()],
  test: {
    environment: "happy-dom",
    globals: true,
    setupFiles: ["./test/setup.ts"],
    include: ["test/perf/**/*.test.{ts,tsx}"],
  },
  resolve: {
    dedupe: ["react", "react-dom"],
  },
});
