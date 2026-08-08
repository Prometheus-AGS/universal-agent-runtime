// For more info, see https://github.com/storybookjs/eslint-plugin-storybook#configuration-flat-config-format
import storybook from "eslint-plugin-storybook";

import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import unicorn from "eslint-plugin-unicorn";
import tseslint from "typescript-eslint";

import {
    flat2FilenameCaseRule,
    flat2RestrictedSyntaxRule,
    legacyFilesFor,
} from "./eslint-flat2-contract.js";

const legacyFlat2StyleFiles = legacyFilesFor("no-restricted-syntax");
const legacyFilenameFiles = legacyFilesFor("unicorn/filename-case");

export default tseslint.config({
    ignores: [
        "dist",
        "coverage/**",
        "test-results/**",
        "../static",
        // This workspace package has its own build/test lifecycle. Linting
        // the product frontend must not traverse its examples, generated
        // skill templates, or independently versioned source tree.
        "packages/**",
        // Storybook build output (Change 25).
        "storybook-static",
        // Deliberately invalid fixtures are exercised by the standalone
        // Flat 2.0 negative gate, never by the product lint traversal.
        "test-fixtures/**",
    ],
}, {
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
        ecmaVersion: 2020,
        globals: globals.browser,
    },
    plugins: {
        "react-hooks": reactHooks,
        "react-refresh": reactRefresh,
        unicorn,
    },
    rules: {
        ...reactHooks.configs.recommended.rules,
        "no-restricted-syntax": flat2RestrictedSyntaxRule,
        // UAR stores expose async load actions that synchronously publish
        // their loading state before awaiting I/O. Calling those actions
        // from mount effects is intentional external-state synchronization,
        // not derived-state mirroring or a cascading-render loop.
        "react-hooks/set-state-in-effect": "off",
        "react-refresh/only-export-components": [
            "warn",
            {
                allowConstantExport: true,
                allowExportNames: [
                    "MemoryContext",
                    "badgeVariants",
                    "buttonVariants",
                    "buttonGroupVariants",
                    "extractAgentConfig",
                    "formatUpdated",
                    "maskedKey",
                    "navigationMenuTriggerStyle",
                    "tabsListVariants",
                    "toggleVariants",
                    "useCarousel",
                    "useComboboxAnchor",
                    "useDb",
                    "useDirection",
                    "useFormField",
                    "useMemoryContext",
                    "useSidebar",
                ],
            },
        ],
        "unicorn/filename-case": flat2FilenameCaseRule,
    },
}, {
    // The standalone gate still scans these exact files without suppression
    // and rejects additions or stale allowlist entries.
    files: legacyFlat2StyleFiles,
    rules: {
        "no-restricted-syntax": "off",
    },
}, {
    files: legacyFilenameFiles,
    rules: {
        "unicorn/filename-case": "off",
    },
}, storybook.configs["flat/recommended"]);
