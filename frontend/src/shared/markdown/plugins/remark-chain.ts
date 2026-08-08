import type { Options } from "react-markdown";
import remarkBreaks from "remark-breaks";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";

/** Shared markdown-syntax plugins for every UAR markdown surface. */
export const remarkChain: NonNullable<Options["remarkPlugins"]> = [
  remarkGfm,
  remarkBreaks,
  remarkMath,
];
