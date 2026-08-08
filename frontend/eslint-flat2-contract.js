import { readFileSync } from "node:fs";

export const flat2RestrictedSyntaxRule = [
    "error",
    {
        selector: "Literal[value=/\\b(border|border-[a-z0-9/-]+|divide-[a-z0-9/-]+|ring-1|shadow-(sm|md|lg|xl|2xl)|backdrop-blur[a-z-]*|bg-gradient-[a-z-]+)\\b/]",
        message: "Flat 2.0: separate by surface fill and spacing, not lines/shadows/blur.",
    },
    {
        selector: "TemplateElement[value.raw=/\\b(border|border-[a-z0-9/-]+|divide-[a-z0-9/-]+|ring-1|shadow-(sm|md|lg|xl|2xl)|backdrop-blur[a-z-]*|bg-gradient-[a-z-]+)\\b/]",
        message: "Flat 2.0: separate by surface fill and spacing, not lines/shadows/blur.",
    },
    {
        selector: "JSXAttribute[name.name='variant'][value.value='outline']",
        message: "Flat 2.0: no outline variants — use the muted filled variant.",
    },
    {
        selector: "JSXAttribute[name.name='variant'] > JSXExpressionContainer > Literal[value='outline']",
        message: "Flat 2.0: no outline variants — use the muted filled variant.",
    },
];

export const flat2FilenameCaseRule = [
    "error",
    {
        case: "kebabCase",
    },
];

const allowlistUrl = new URL("../scripts/frontend-flat2-style-allowlist.txt", import.meta.url);

export function legacyFilesFor(ruleId) {
    const prefix = "frontend/";
    return [...new Set(
        readFileSync(allowlistUrl, "utf8")
            .split(/\r?\n/)
            .map((line) => line.trim())
            .filter((line) => line && !line.startsWith("#"))
            .filter((line) => line.split("|", 2)[1] === ruleId)
            .map((line) => line.split("|", 1)[0])
            .filter((file) => file.startsWith(prefix))
            .map((file) => file.slice(prefix.length)),
    )].sort();
}
