import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "node:path";
import { fileURLToPath } from 'node:url';
import { storybookTest } from '@storybook/addon-vitest/vitest-plugin';
import { playwright } from '@vitest/browser-playwright';
const dirname = typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url));

// More info at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon
export default defineConfig({
  plugins: [react()],
  server: {
    fs: {
      // Workspace dependencies are root-hoisted; tests and browser stories
      // must be able to load their package assets through Vite.
      allow: [path.resolve(dirname, "..")],
    },
  },
  resolve: {
    dedupe: ["react", "react-dom", "use-sync-external-store", "zustand"],
    alias: {
      "@": path.resolve(__dirname, "./src")
    }
  },
  test: {
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov", "html"],
      reportsDirectory: "coverage",
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/**/*.test.{ts,tsx}", "src/test/**", "**/*.d.ts"],
      thresholds: {
        lines: 60,
        statements: 60,
        functions: 60,
        branches: 60
      }
    },
    projects: [{
      extends: true,
      test: {
        name: 'unit',
        environment: "happy-dom",
        globals: true,
        setupFiles: ["./src/test/setup.ts"],
        include: ["src/**/*.test.{ts,tsx}"],
        exclude: ["**/node_modules/**", "**/packages/**", "**/dist/**", "e2e/**"]
      }
    }, {
      extends: true,
      plugins: [
      // The plugin will run tests for the stories defined in your Storybook config
      // See options at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon#storybooktest
      storybookTest({
        configDir: path.join(dirname, '.storybook')
      })],
      test: {
        name: 'storybook',
        browser: {
          enabled: true,
          headless: true,
          provider: playwright({}),
          instances: [{
            browser: 'chromium'
          }]
        }
      }
    }]
  }
});
