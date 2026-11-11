import react from "@vitejs/plugin-react";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";
import { compileA2uiTestDecorators } from "../a2ui-core/vitest-decorators.ts";
export default defineConfig({ plugins: [react(), svelte(), compileA2uiTestDecorators()], resolve: { conditions: ["browser"] }, test: { environment: "happy-dom", setupFiles: ["./test/setup.ts"], exclude: ["dist/**", "node_modules/**"] } });
