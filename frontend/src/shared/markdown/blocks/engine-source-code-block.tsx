import { type FC } from "react";

interface EngineSourceCodeBlockProps {
  language: string;
  source: string;
  status: string;
  wrap?: boolean;
}

/** Source fallback kept inside the lazy engine subtree to avoid an entry-chunk cycle. */
export const EngineSourceCodeBlock: FC<EngineSourceCodeBlockProps> = ({
  language,
  source,
  status,
  wrap = false,
}) => (
  <div className="my-3 overflow-hidden rounded-lg bg-muted/30" data-markdown-source-fallback>
    <div className="flex items-center justify-between gap-3 bg-muted/60 px-3 py-2 font-mono text-[11px] text-muted-foreground">
      <span>{language.toLowerCase()}</span>
      <span>{status}</span>
    </div>
    <pre className={`overflow-x-auto p-4 font-mono text-sm leading-relaxed${wrap ? " whitespace-pre-wrap break-words" : ""}`}>
      <code>{source}</code>
    </pre>
  </div>
);
