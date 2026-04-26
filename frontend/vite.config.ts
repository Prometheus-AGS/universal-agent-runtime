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
        manualChunks: {
          "vendor-react": ["react", "react-dom", "react-router-dom"],
          "vendor-assistant": ["@assistant-ui/react", "@assistant-ui/react-markdown"],
          "vendor-query": ["@tanstack/react-query", "zustand", "immer"],
          "vendor-hljs": ["highlight.js"],
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
