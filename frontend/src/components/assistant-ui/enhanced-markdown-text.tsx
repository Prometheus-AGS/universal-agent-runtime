import {
  MarkdownTextPrimitive,
  unstable_memoizeMarkdownComponents as memoizeMarkdownComponents,
  useIsMarkdownCodeBlock,
} from "@assistant-ui/react-markdown";
import remarkGfm from "remark-gfm";
import { type FC, memo, useEffect, useRef } from "react";
import { cn } from "@/lib/utils";
import hljs from "highlight.js";

import "@assistant-ui/react-markdown/styles/dot.css";

// Inline code
const InlineCode: FC<React.HTMLAttributes<HTMLElement>> = ({ className, ...props }) => (
  <code className={cn("rounded-md border border-border/50 bg-muted/50 px-1.5 py-0.5 font-mono text-[0.85em]", className)} {...props} />
);

// Highlighted code block
const CodeBlock: FC<{ code: string; language?: string }> = ({ code, language }) => {
  const ref = useRef<HTMLElement>(null);
  useEffect(() => {
    if (!ref.current) return;
    ref.current.removeAttribute("data-highlighted");
    if (language && hljs.getLanguage(language)) {
      const result = hljs.highlight(code, { language });
      ref.current.innerHTML = result.value;
    } else {
      const result = hljs.highlightAuto(code);
      ref.current.innerHTML = result.value;
    }
  }, [code, language]);
  return (
    <div className="my-3 overflow-hidden rounded-lg border border-border/50">
      {language && (
        <div className="flex items-center justify-between border-b border-border/50 bg-muted/50 px-3 py-1.5">
          <span className="font-mono text-[10px] uppercase tracking-widest text-muted-foreground">{language}</span>
        </div>
      )}
      <pre className="overflow-x-auto p-0"><code ref={ref} className="hljs block p-4 text-[13px] leading-relaxed">{code}</code></pre>
    </div>
  );
};

const EnhancedCodeBlock: FC<React.HTMLAttributes<HTMLElement> & { "data-language"?: string }> = ({ children, "data-language": language, ...props }) => {
  const isCodeBlock = useIsMarkdownCodeBlock();
  const code = typeof children === "string" ? children : String(children ?? "");
  if (!isCodeBlock) return <InlineCode {...props}>{children}</InlineCode>;
  return <CodeBlock code={code} language={language} />;
};

const enhancedComponents = memoizeMarkdownComponents({
  h1: ({ className, ...props }) => <h1 className={cn("mb-2 font-display font-semibold text-base first:mt-0 last:mb-0", className)} {...props} />,
  h2: ({ className, ...props }) => <h2 className={cn("mt-3 mb-1.5 font-display font-semibold text-sm first:mt-0 last:mb-0", className)} {...props} />,
  h3: ({ className, ...props }) => <h3 className={cn("mt-2.5 mb-1 font-display font-semibold text-sm first:mt-0 last:mb-0", className)} {...props} />,
  p: ({ className, ...props }) => <p className={cn("my-2.5 leading-relaxed first:mt-0 last:mb-0", className)} {...props} />,
  a: ({ className, ...props }) => <a className={cn("text-primary underline underline-offset-2 hover:text-primary/80", className)} target="_blank" rel="noopener noreferrer" {...props} />,
  blockquote: ({ className, ...props }) => <blockquote className={cn("my-2.5 border-l-2 border-primary/30 pl-3 text-muted-foreground italic", className)} {...props} />,
  ul: ({ className, ...props }) => <ul className={cn("my-2 ml-4 list-disc marker:text-muted-foreground/60 [&>li]:mt-1", className)} {...props} />,
  ol: ({ className, ...props }) => <ol className={cn("my-2 ml-4 list-decimal marker:text-muted-foreground/60 [&>li]:mt-1", className)} {...props} />,
  li: ({ className, ...props }) => <li className={cn("leading-relaxed", className)} {...props} />,
  table: ({ className, ...props }) => <div className="my-3 overflow-x-auto rounded-lg border border-border/50"><table className={cn("w-full border-separate border-spacing-0", className)} {...props} /></div>,
  th: ({ className, ...props }) => <th className={cn("bg-muted/50 px-3 py-2 text-left font-mono text-[11px] font-medium uppercase tracking-widest text-muted-foreground", className)} {...props} />,
  td: ({ className, ...props }) => <td className={cn("border-b border-border/30 px-3 py-2 text-left text-sm", className)} {...props} />,
  tr: ({ className, ...props }) => <tr className={cn("transition-colors hover:bg-muted/20 [&:last-child>td]:border-b-0", className)} {...props} />,
  hr: ({ className, ...props }) => <hr className={cn("my-4 border-border/30", className)} {...props} />,
  pre: ({ className, ...props }) => <pre className={cn("overflow-x-auto", className)} {...props} />,
  code: EnhancedCodeBlock as typeof InlineCode,
});

const EnhancedMarkdownTextImpl: FC = () => (
  <MarkdownTextPrimitive
    remarkPlugins={[remarkGfm]}
    className="aui-md prose-sm max-w-none text-foreground"
    components={enhancedComponents}
  />
);

export const EnhancedMarkdownText = memo(EnhancedMarkdownTextImpl);
