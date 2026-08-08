import type { Options } from "react-markdown";
import rehypeKatex from "rehype-katex";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { markdownSanitizeSchema } from "./sanitize-schema";

/**
 * Shared HTML-stage plugins. Raw parsing and sanitization are one ordered unit:
 * nothing may run between them.
 */
export const rehypeChain: NonNullable<Options["rehypePlugins"]> = [
  rehypeRaw,
  [rehypeSanitize, markdownSanitizeSchema],
  [rehypeKatex, { throwOnError: false, strict: "ignore" }],
];
