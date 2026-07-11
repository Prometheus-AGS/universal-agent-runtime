import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";

export default tseslint.config(
    {
        ignores: [
            "dist",
            "../static",
            // This workspace package has its own build/test lifecycle. Linting
            // the product frontend must not traverse its examples, generated
            // skill templates, or independently versioned source tree.
            "packages/**",
        ],
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
            "react-refresh": reactRefresh,
        },
        rules: {
            ...reactHooks.configs.recommended.rules,
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
        },
    }
);
