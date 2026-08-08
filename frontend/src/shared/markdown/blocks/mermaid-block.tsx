import mermaid from "mermaid";
import { type FC, useEffect, useId, useState } from "react";
import { sanitizeRawSvg } from "../plugins/sanitize-raw-svg";
import { EngineSourceCodeBlock } from "./engine-source-code-block";

interface MermaidBlockProps {
  source: string;
  theme: "dark" | "high-contrast" | "light";
}

const cssColor = (name: string, fallback: string): string => {
  if (typeof document === "undefined") return fallback;
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
};

const mermaidThemeVariables = () => ({
  background: cssColor("--color-background", "#111318"),
  primaryColor: cssColor("--color-card", "#20232b"),
  primaryTextColor: cssColor("--color-foreground", "#f4f4f5"),
  primaryBorderColor: cssColor("--color-card", "#20232b"),
  lineColor: cssColor("--color-muted-foreground", "#9ca3af"),
  secondaryColor: cssColor("--color-muted", "#2a2e38"),
  tertiaryColor: cssColor("--color-primary", "#f97316"),
});

/** Finalized Mermaid syntax rendered under the strict untrusted-diagram policy. */
export const MermaidBlock: FC<MermaidBlockProps> = ({ source, theme }) => {
  const reactId = useId();
  const [svg, setSvg] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let current = true;
    const renderId = `uar-mermaid-${reactId.replaceAll(":", "")}`;

    setSvg(null);
    setError(null);

    mermaid.initialize({
      startOnLoad: false,
      securityLevel: "strict",
      secure: ["securityLevel", "startOnLoad", "theme", "themeVariables"],
      suppressErrorRendering: true,
      theme: "base",
      themeVariables: mermaidThemeVariables(),
    });

    void mermaid.render(renderId, source).then(({ svg: renderedSvg }) => {
      if (!current) return;
      const sanitized = sanitizeRawSvg(renderedSvg);
      if (!sanitized.includes("<svg")) {
        setError("Diagram output could not be sanitized");
        return;
      }
      setSvg(sanitized);
    }).catch((reason: unknown) => {
      if (!current) return;
      const firstLine = reason instanceof Error ? reason.message.split("\n", 1)[0] : "Diagram could not be rendered";
      setError(firstLine || "Diagram could not be rendered");
    });

    return () => {
      current = false;
    };
  }, [reactId, source, theme]);

  if (error) {
    return <EngineSourceCodeBlock source={source} language="mermaid" status={error} />;
  }

  if (svg === null) {
    return <EngineSourceCodeBlock source={source} language="mermaid" status="Loading diagram preview" />;
  }

  return (
    <figure className="my-3 overflow-hidden rounded-lg bg-muted/30" data-mermaid-block data-mermaid-theme={theme}>
      <div className="bg-muted/60 px-3 py-2 font-mono text-[11px] text-muted-foreground">mermaid</div>
      <div
        role="img"
        aria-label="Mermaid diagram"
        className="overflow-x-auto p-4 [&_svg]:mx-auto [&_svg]:h-auto [&_svg]:max-w-full"
        // Mermaid creates the SVG; strict configuration and DOMPurify own this insertion boundary.
        dangerouslySetInnerHTML={{ __html: svg }}
      />
      <details className="bg-muted/40 px-3 py-2 text-sm">
        <summary className="cursor-pointer font-mono text-[11px] text-muted-foreground focus-visible:outline-[3px] focus-visible:outline-offset-2 focus-visible:outline-primary">
          Diagram source
        </summary>
        <pre className="mt-2 overflow-x-auto whitespace-pre-wrap p-2 font-mono text-xs leading-relaxed"><code>{source}</code></pre>
      </details>
    </figure>
  );
};
