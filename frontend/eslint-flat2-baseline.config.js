import unicorn from "eslint-plugin-unicorn";
import tseslint from "typescript-eslint";

import {
    flat2FilenameCaseRule,
    flat2RestrictedSyntaxRule,
} from "./eslint-flat2-contract.js";

export default tseslint.config({
    ignores: [
        "node_modules/**",
        "dist/**",
        "coverage/**",
        "test-results/**",
        "storybook-static/**",
        "packages/**",
    ],
}, {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
        parser: tseslint.parser,
        parserOptions: {
            ecmaFeatures: {
                jsx: true,
            },
            sourceType: "module",
        },
    },
    plugins: {
        unicorn,
    },
    rules: {
        "no-restricted-syntax": flat2RestrictedSyntaxRule,
        "unicorn/filename-case": flat2FilenameCaseRule,
    },
});
