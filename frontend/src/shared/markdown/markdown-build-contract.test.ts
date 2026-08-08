import { readFileSync } from "node:fs";
import { describe, expect, test } from "vitest";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");

describe("markdown build contract", () => {
  test("keeps engines behind block-level dynamic imports", () => {
    const components = read("./markdown-components.tsx");
    const shikiEntry = read("./blocks/vendor-shiki.ts");
    const mermaidEntry = read("./blocks/vendor-mermaid.ts");
    const codeBlock = read("./blocks/code-block.tsx");
    const mermaidBlock = read("./blocks/mermaid-block.tsx");

    expect(components).toContain('import("./blocks/vendor-shiki")');
    expect(components).toContain('import("./blocks/vendor-mermaid")');
    expect(components).not.toMatch(/from ["'](?:mermaid|shiki)/u);
    expect(shikiEntry).toContain('from "./code-block"');
    expect(mermaidEntry).toContain('from "./mermaid-block"');
    expect(codeBlock).toContain('from "shiki/bundle/full"');
    expect(mermaidBlock).toContain('from "mermaid"');
  });

  test("does not force either lazy engine into a package-wide vendor group", () => {
    const viteConfig = read("../../../vite.config.ts");

    expect(viteConfig).not.toMatch(/name:\s*["']vendor-(?:mermaid|shiki)["']/u);
    expect(viteConfig).not.toMatch(/node_modules.*(?:mermaid|shiki)/u);
  });

  test("ships a recursive production-manifest gate for both named dynamic entries", () => {
    const checker = read("../../../../scripts/check-markdown-lazy-chunks.mjs");
    const viteConfig = read("../../../vite.config.ts");
    const graphPlugin = read("../../../build/markdown-engine-graph-plugin.ts");

    expect(checker).toContain('"src/shared/markdown/blocks/vendor-shiki.ts"');
    expect(checker).toContain('"src/shared/markdown/blocks/vendor-mermaid.ts"');
    expect(checker).toContain("Engine graph static record");
    expect(checker).toContain("Engine graph static closure mismatch");
    expect(checker).toContain("manifestStaticFiles");
    expect(checker).toContain("staticReachable");
    expect(checker).toContain("forbiddenStatic");
    expect(checker).toContain("missingDynamic");
    expect(checker).toContain("invalidNames");
    expect(checker).toContain("absoluteModuleIds");
    expect(viteConfig).toContain("markdownEngineGraphPlugin()");
    expect(graphPlugin).toContain("chunk.modules");
    expect(graphPlugin).toContain("packageRelativeModuleId");
    expect(graphPlugin).toContain('fileName: ".vite/markdown-engine-graph.json"');
  });

  test("keeps KaTeX CSS under the shared entry only", () => {
    const bubble = read("./markdown-bubble.tsx");
    const components = read("./markdown-components.tsx");
    const codeBlock = read("./blocks/code-block.tsx");
    const mermaidBlock = read("./blocks/mermaid-block.tsx");

    expect(bubble.match(/katex\/dist\/katex\.min\.css/gu)).toHaveLength(1);
    expect(`${components}${codeBlock}${mermaidBlock}`).not.toMatch(/katex|https?:\/\//u);
  });
});
