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
    rollupOptions: {
      output: {
        // Vite 8 (Rolldown) removed the object form of manualChunks; the
        // function form is deprecated but still supported. Same chunk
        // groupings as before, matched by module id instead of package name.
        manualChunks(id) {
          if (/node_modules\/(react|react-dom|react-router-dom)\//.test(id)) {
            return "vendor-react";
          }
          if (/node_modules\/@assistant-ui\/(react|react-markdown)\//.test(id)) {
            return "vendor-assistant";
          }
          if (/node_modules\/(@tanstack\/react-query|zustand|immer)\//.test(id)) {
            return "vendor-query";
          }
          if (/node_modules\/highlight\.js\//.test(id)) {
            return "vendor-hljs";
          }
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
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
