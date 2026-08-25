import { MarkdownTextPrimitive } from "@assistant-ui/react-markdown";
import { useAuiState } from "@assistant-ui/react";
import { type FC, memo } from "react";
import ReactMarkdown from "react-markdown";
import { useTheme } from "@/hooks/use-theme";
import { cn } from "@/lib/utils";
import { markdownComponents, MarkdownRenderPhaseProvider } from "./markdown-components";
import { rehypeChain } from "./plugins/rehype-chain";
import { remarkChain } from "./plugins/remark-chain";

import "@assistant-ui/react-markdown/styles/dot.css";
import "katex/dist/katex.min.css";

interface MarkdownBubbleProps {
  /** Explicit markdown for previews and read-only surfaces. Omit inside an assistant-ui message part. */
  source?: string;
  /** Supplied by assistant-ui's Text component contract; the primitive reads it from context. */
  text?: string;
  className?: string;
}

const BASE_CLASS_NAME = "aui-md prose-sm max-w-none text-foreground";

const MarkdownBubbleImpl: FC<MarkdownBubbleProps> = ({ source, className }) => {
  const resolvedClassName = cn(BASE_CLASS_NAME, className);
  const messageStatus = useAuiState((state) => state.optional.message?.status?.type);
  const { resolved: resolvedTheme } = useTheme();
  const isRunning = messageStatus === "running";
  const phase = source === undefined && isRunning ? "streaming" : "finalized";

  if (source !== undefined) {
    return (
      <MarkdownRenderPhaseProvider phase={phase} theme={resolvedTheme}>
        <div className={resolvedClassName}>
          <ReactMarkdown
            remarkPlugins={remarkChain}
            rehypePlugins={rehypeChain}
            components={markdownComponents}
          >
            {source}
          </ReactMarkdown>
        </div>
      </MarkdownRenderPhaseProvider>
    );
  }

  return (
    <MarkdownRenderPhaseProvider phase={phase} theme={resolvedTheme}>
      <MarkdownTextPrimitive
        remarkPlugins={remarkChain}
        rehypePlugins={rehypeChain}
        className={resolvedClassName}
        components={markdownComponents}
        defer
      />
    </MarkdownRenderPhaseProvider>
  );
};

/** The sole public renderer for markdown-derived UAR text. */
export const MarkdownBubble: FC<MarkdownBubbleProps> = memo(MarkdownBubbleImpl);
