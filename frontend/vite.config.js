import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";
import { markdownEngineGraphPlugin } from "./build/markdown-engine-graph-plugin";
export default defineConfig({
    plugins: [react(), tailwindcss(), markdownEngineGraphPlugin()],
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
                                if (/node_modules\/@electric-sql\/pglite\//.test(moduleId)) {
                                    return "vendor-pglite";
                                }
                                if (moduleId.includes("packages/prometheus-entity-management/")) {
                                    return "vendor-entities";
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
        fs: {
            // pnpm hoists dependencies into the repo-root node_modules/.pnpm
            // store; Vite's default fs.allow is scoped to this package alone
            // and denies (403) requests for assets like @electric-sql/pglite's
            // .wasm files that live one level up. Explicitly allow the repo
            // root (one level above this package).
            allow: [path.resolve(__dirname, "..")],
        },
        proxy: {
            "/api": {
                target: process.env.UAR_BACKEND_URL ?? "http://127.0.0.1:1906",
                changeOrigin: true,
            },
            "/healthz": {
                target: process.env.UAR_BACKEND_URL ?? "http://127.0.0.1:1906",
                changeOrigin: true,
            },
            "/readyz": {
                target: process.env.UAR_BACKEND_URL ?? "http://127.0.0.1:1906",
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
