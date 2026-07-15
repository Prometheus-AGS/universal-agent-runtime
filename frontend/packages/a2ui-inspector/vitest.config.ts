import react from "@vitejs/plugin-react";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";
export default defineConfig({ plugins: [react(), svelte()], resolve: { conditions: ["browser"] }, test: { environment: "happy-dom", setupFiles: ["./test/setup.ts"], exclude: ["dist/**", "node_modules/**"] } });
