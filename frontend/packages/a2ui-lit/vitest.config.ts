import { defineConfig } from "vitest/config";
import { compileA2uiTestDecorators } from "../a2ui-core/vitest-decorators.ts";
export default defineConfig({ plugins: [compileA2uiTestDecorators()], test: { environment: "happy-dom", exclude: ["dist/**", "node_modules/**"] } });
