import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";
import { compileA2uiTestDecorators } from "../a2ui-core/vitest-decorators.ts";
export default defineConfig({ plugins: [svelte(), compileA2uiTestDecorators()], resolve: { conditions: ["browser"] }, test: { environment: "happy-dom", exclude: ["dist/**", "node_modules/**"] } });
