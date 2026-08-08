import { type FC } from "react";
import { cn } from "@/lib/utils";

interface SourceCodeBlockProps {
  source: string;
  language?: string;
  status?: string;
  wrap?: boolean;
}

/** Stable escaped-source presentation used during streaming, loading, and failures. */
export const SourceCodeBlock: FC<SourceCodeBlockProps> = ({
  source,
  language = "text",
  status,
  wrap = false,
}) => (
  <div className="my-3 overflow-hidden rounded-lg bg-muted/30" data-markdown-source-fallback>
    <div className="flex items-center justify-between gap-3 bg-muted/60 px-3 py-2 font-mono text-[11px] text-muted-foreground">
      <span>{language.toLowerCase()}</span>
      {status ? <span>{status}</span> : null}
    </div>
    <pre className={cn("overflow-x-auto p-4 font-mono text-sm leading-relaxed", wrap && "whitespace-pre-wrap break-words")}>
      <code>{source}</code>
    </pre>
  </div>
);
