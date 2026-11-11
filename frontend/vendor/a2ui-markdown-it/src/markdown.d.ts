import type { MarkdownRenderer, MarkdownRendererOptions, MarkdownRendererTagClassMap } from '@a2ui/web_core';
export type { MarkdownRenderer, MarkdownRendererOptions, MarkdownRendererTagClassMap };
/**
 * A Markdown to HTML renderer using markdown-it and dompurify.
 * @param value The markdown code to render.
 * @param options Options for the markdown renderer.
 * @returns A promise that resolves to the rendered HTML as a string.
 */
export declare function renderMarkdown(value: string, options?: MarkdownRendererOptions): Promise<string>;
//# sourceMappingURL=markdown.d.ts.map