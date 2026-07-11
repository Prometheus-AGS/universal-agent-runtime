import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  optimizeDeps: {
    exclude: ["@electric-sql/pglite"],
  },
  build: {
    outDir: "../static",
    emptyOutDir: true,
    chunkSizeWarningLimit: 1100,
    rolldownOptions: {
      output: {
        // Vite 8 (Rolldown) removed the object form of manualChunks, and the
        // function form is itself deprecated in favor of Rolldown's native
        // codeSplitting API. Same chunk groupings as before, matched by
        // module id instead of package name.
        codeSplitting: {
          groups: [
            {
              name(moduleId) {
                if (/node_modules\/(react|react-dom|react-router-dom)\//.test(moduleId)) {
                  return "vendor-react";
                }
                if (/node_modules\/@assistant-ui\/(react|react-markdown)\//.test(moduleId)) {
                  return "vendor-assistant";
                }
                if (/node_modules\/(@tanstack\/react-query|zustand|immer)\//.test(moduleId)) {
                  return "vendor-query";
                }
                if (/node_modules\/highlight\.js\//.test(moduleId)) {
                  return "vendor-hljs";
                }
              },
            },
          ],
        },
      },
    },
  },
  server: {
    host: "::",
    port: 8080,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:6565",
        changeOrigin: true,
      },
      "/healthz": {
        target: "http://127.0.0.1:6565",
        changeOrigin: true,
      },
      "/readyz": {
        target: "http://127.0.0.1:6565",
        changeOrigin: true,
      },
    },
  },
  resolve: {
    // Linked workspace packages must use the host application's singleton
    // React and Zustand instances. Without dedupe, a standalone package
    // install can produce invalid-hook-call failures in development/tests.
    dedupe: ["react", "react-dom", "use-sync-external-store", "zustand"],
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
