/* eslint-disable react-refresh/only-export-components -- assistant-ui requires one stable exported component map. */
import {
  unstable_memoizeMarkdownComponents as memoizeMarkdownComponents,
  useIsMarkdownCodeBlock,
} from "@assistant-ui/react-markdown";
import {
  type ComponentPropsWithoutRef,
  createContext,
  lazy,
  Suspense,
  useContext,
} from "react";
import { cn } from "@/lib/utils";
import { LazyMarkdownBlockBoundary } from "./blocks/lazy-markdown-block-boundary";
import { SourceCodeBlock } from "./blocks/source-code-block";

const MarkdownCodeBlockContext = createContext(false);
interface MarkdownRenderState {
  phase: "finalized" | "streaming";
  theme: "dark" | "high-contrast" | "light";
}

const MarkdownRenderPhaseContext = createContext<MarkdownRenderState>({
  phase: "finalized",
  theme: "dark",
});

const LazyCodeBlock = lazy(async () => {
  const module = await import("./blocks/vendor-shiki");
  return { default: module.CodeBlock };
});

const LazyMermaidBlock = lazy(async () => {
  const module = await import("./blocks/vendor-mermaid");
  return { default: module.MermaidBlock };
});

interface MarkdownRenderPhaseProviderProps {
  children: React.ReactNode;
  phase: "finalized" | "streaming";
  theme: "dark" | "high-contrast" | "light";
}

/** Carries assistant message finalization without introducing store or service dependencies. */
export const MarkdownRenderPhaseProvider = ({ children, phase, theme }: MarkdownRenderPhaseProviderProps) => (
  <MarkdownRenderPhaseContext.Provider value={{ phase, theme }}>{children}</MarkdownRenderPhaseContext.Provider>
);

const MarkdownPre = ({ children }: ComponentPropsWithoutRef<"pre">) => (
  <MarkdownCodeBlockContext.Provider value>
    {children}
  </MarkdownCodeBlockContext.Provider>
);

const MarkdownCode = ({ children, className, ...props }: ComponentPropsWithoutRef<"code">) => {
  const isAssistantCodeBlock = useIsMarkdownCodeBlock();
  const isSourceCodeBlock = useContext(MarkdownCodeBlockContext);
  const renderState = useContext(MarkdownRenderPhaseContext);
  const isCodeBlock = isAssistantCodeBlock || isSourceCodeBlock;

  if (isCodeBlock) {
    const language = /(?:^|\s)language-([^\s]+)/u.exec(className ?? "")?.[1] ?? "text";
    const source = String(children).replace(/\n$/u, "");
    const fallback = (
      <SourceCodeBlock
        source={source}
        language={language}
        status={renderState.phase === "streaming" ? "Preview available when response finishes" : "Loading preview"}
      />
    );

    if (renderState.phase === "streaming") return fallback;

    return (
      <LazyMarkdownBlockBoundary language={language} resetKey={`${language}:${source}`} source={source}>
        <Suspense fallback={fallback}>
          {language.toLowerCase() === "mermaid"
            ? <LazyMermaidBlock source={source} theme={renderState.theme} />
            : <LazyCodeBlock language={language} source={source} theme={renderState.theme} />}
        </Suspense>
      </LazyMarkdownBlockBoundary>
    );
  }

  return (
    <code
      className={cn("rounded-md bg-muted px-1.5 py-0.5 font-mono text-[0.85em]", className)}
      {...props}
    >
      {children}
    </code>
  );
};

/** Shared presentation map for assistant-ui and explicit-source markdown. */
export const markdownComponents = memoizeMarkdownComponents({
  h1: ({ className, ...props }) => (
    <h1 className={cn("mb-2 font-display text-base font-semibold first:mt-0 last:mb-0", className)} {...props} />
  ),
  h2: ({ className, ...props }) => (
    <h2 className={cn("mb-1.5 mt-3 font-display text-sm font-semibold first:mt-0 last:mb-0", className)} {...props} />
  ),
  h3: ({ className, ...props }) => (
    <h3 className={cn("mb-1 mt-2.5 font-display text-sm font-semibold first:mt-0 last:mb-0", className)} {...props} />
  ),
  p: ({ className, ...props }) => (
    <p className={cn("my-2.5 leading-relaxed first:mt-0 last:mb-0", className)} {...props} />
  ),
  a: ({ className, ...props }) => (
    <a
      {...props}
      className={cn("text-primary underline underline-offset-2 hover:text-primary/80", className)}
      target="_blank"
      rel="noopener noreferrer"
    />
  ),
  blockquote: ({ className, ...props }) => (
    <blockquote
      className={cn("my-2.5 rounded-md bg-muted/40 px-3 py-2 text-muted-foreground italic", className)}
      {...props}
    />
  ),
  ul: ({ className, ...props }) => (
    <ul className={cn("my-2 ml-4 list-disc marker:text-muted-foreground/60 [&>li]:mt-1", className)} {...props} />
  ),
  ol: ({ className, ...props }) => (
    <ol className={cn("my-2 ml-4 list-decimal marker:text-muted-foreground/60 [&>li]:mt-1", className)} {...props} />
  ),
  li: ({ className, ...props }) => <li className={cn("leading-relaxed", className)} {...props} />,
  table: ({ className, ...props }) => (
    <div className="my-3 overflow-x-auto rounded-lg bg-muted/30 p-1">
      <table className={cn("w-full", className)} {...props} />
    </div>
  ),
  th: ({ className, ...props }) => (
    <th
      className={cn("bg-muted/70 px-3 py-2 text-left font-mono text-[11px] font-medium uppercase tracking-widest text-muted-foreground", className)}
      {...props}
    />
  ),
  td: ({ className, ...props }) => <td className={cn("px-3 py-2 text-left text-sm", className)} {...props} />,
  tr: ({ className, ...props }) => (
    <tr className={cn("transition-colors even:bg-muted/30 hover:bg-muted/50", className)} {...props} />
  ),
  hr: () => <div className="my-5 h-3" role="separator" />,
  pre: MarkdownPre,
  code: MarkdownCode,
});
