import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";

/**
 * This package has its own build/test/lint lifecycle (see
 * `frontend/eslint.config.js`, which deliberately ignores `packages/**`).
 * Mirrors the root config's rule set minus `react-refresh` (irrelevant —
 * this is a library package, not a Vite HMR entry point).
 */
export default tseslint.config(
  {
    ignores: ["node_modules", "dist"],
  },
  {
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    plugins: {
      "react-hooks": reactHooks,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
    },
  },
);
