import { type CSSProperties, type FC, useEffect, useState } from "react";
import {
  bundledLanguages,
  codeToTokens,
  type BundledLanguage,
} from "shiki/bundle/full";
import { EngineSourceCodeBlock } from "./engine-source-code-block";

interface CodeBlockProps {
  language: string;
  source: string;
  theme: "dark" | "high-contrast" | "light";
}

type HighlightedCode = Awaited<ReturnType<typeof codeToTokens>>;

const copySource = async (source: string): Promise<boolean> => {
  if (!navigator.clipboard) return false;

  try {
    await navigator.clipboard.writeText(source);
    return true;
  } catch {
    return false;
  }
};

const tokenStyle = (color: string | undefined): CSSProperties | undefined =>
  color ? { color } : undefined;

const resolveLanguage = (language: string): BundledLanguage | "text" | null => {
  const normalized = language.toLowerCase() || "text";
  if (normalized === "text" || normalized === "txt" || normalized === "plaintext") {
    return "text";
  }
  return normalized in bundledLanguages ? normalized as BundledLanguage : null;
};

/** Finalized fenced code rendered from Shiki token data, never highlighter HTML. */
export const CodeBlock: FC<CodeBlockProps> = ({ language, source, theme }) => {
  const [highlighted, setHighlighted] = useState<HighlightedCode | null>(null);
  const [failed, setFailed] = useState(false);
  const [wrap, setWrap] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const resolvedLanguage = resolveLanguage(language);
    let current = true;

    setHighlighted(null);
    setFailed(false);
    setCopied(false);

    if (resolvedLanguage === null) {
      setFailed(true);
      return () => {
        current = false;
      };
    }

    void codeToTokens(source, {
      lang: resolvedLanguage,
      theme: theme === "light" ? "github-light" : "github-dark",
    }).then((result) => {
      if (current) setHighlighted(result);
    }).catch(() => {
      if (current) setFailed(true);
    });

    return () => {
      current = false;
    };
  }, [language, source, theme]);

  if (failed) {
    return (
      <EngineSourceCodeBlock
        source={source}
        language={language}
        status="Syntax preview unavailable; showing source"
        wrap={wrap}
      />
    );
  }

  if (highlighted === null) {
    return <EngineSourceCodeBlock source={source} language={language} status="Loading syntax preview" wrap={wrap} />;
  }

  const showLineNumbers = highlighted.tokens.length > 8;

  return (
    <div className="my-3 overflow-hidden rounded-lg bg-muted/30" data-shiki-code-block data-shiki-theme={highlighted.themeName}>
      <div className="flex items-center justify-between gap-3 bg-muted/60 px-3 py-2">
        <span className="font-mono text-[11px] text-muted-foreground">{language.toLowerCase() || "text"}</span>
        <div className="flex items-center gap-1">
          <button
            type="button"
            className="rounded-md bg-background/60 px-2 py-1 font-mono text-[11px] text-muted-foreground hover:bg-background focus-visible:outline-[3px] focus-visible:outline-offset-2 focus-visible:outline-primary"
            aria-pressed={wrap}
            onClick={() => setWrap((value) => !value)}
          >
            {wrap ? "No wrap" : "Wrap"}
          </button>
          <button
            type="button"
            className="rounded-md bg-background/60 px-2 py-1 font-mono text-[11px] text-muted-foreground hover:bg-background focus-visible:outline-[3px] focus-visible:outline-offset-2 focus-visible:outline-primary"
            onClick={() => {
              void copySource(source).then(setCopied);
            }}
          >
            {copied ? "Copied" : "Copy"}
          </button>
        </div>
      </div>
      <pre className={`overflow-x-auto p-4 font-mono text-sm leading-relaxed${wrap ? " whitespace-pre-wrap break-words" : ""}`}>
        <code>
          {highlighted.tokens.map((line, lineIndex) => (
            <span
              key={`${lineIndex}-${line[0]?.offset ?? 0}`}
              className={`block min-h-[1lh]${showLineNumbers ? " grid grid-cols-[3ch_1fr] gap-3" : ""}`}
            >
              {showLineNumbers ? (
                <span aria-hidden="true" className="select-none text-right text-muted-foreground/60">
                  {lineIndex + 1}
                </span>
              ) : null}
              <span>
                {line.map((token) => (
                  <span key={`${token.offset}-${token.content}`} style={tokenStyle(token.color)}>{token.content}</span>
                ))}
              </span>
            </span>
          ))}
        </code>
      </pre>
    </div>
  );
};
