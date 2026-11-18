import { useEffect, useId, useMemo, useRef, useState } from "react";
import { Check, Copy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Disclosure } from "./disclosure";
import { needsClamp, outputText, splitThinking } from "./run-model";

type CopyState = "idle" | "copied" | "failed";
const COPY_NOTICE_MS = 2_000;

/**
 * An attempt's output or an artifact's content. A string shows as text, anything else as formatted JSON.
 * Long output is clamped to about three lines with Show more; copy always takes the complete original text. `<think>` blocks
 * are folded behind a disclosure. Expansion state is local, so it survives refreshes.
 */
export function OutputView({ label, value }: { label: string; value: unknown }) {
  const text = useMemo(() => outputText(value), [value]);
  const isText = typeof value === "string";
  const { answer, thoughts } = useMemo(() => (isText ? splitThinking(text) : { answer: text, thoughts: [] }), [isText, text]);
  const clampable = useMemo(() => needsClamp(answer), [answer]);
  const [expanded, setExpanded] = useState(false);
  const bodyId = useId();
  const [copy, setCopy] = useState<CopyState>("idle");
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    [],
  );

  const copyText = async () => {
    let next: CopyState = "copied";
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      next = "failed";
    }
    setCopy(next);
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => setCopy("idle"), COPY_NOTICE_MS);
  };

  return (
    <div className="min-w-0 space-y-1.5">
      <p className="text-xs font-medium text-muted-foreground">{label}</p>
      {answer === "" ? (
        <p className="text-sm text-muted-foreground">{thoughts.length > 0 ? "Only reasoning was returned." : "Empty."}</p>
      ) : (
        <div
          id={bodyId}
          role={expanded && clampable ? "region" : undefined}
          aria-label={expanded && clampable ? label : undefined}
          tabIndex={expanded && clampable ? 0 : undefined}
          className={cn(
            "min-w-0 rounded-md bg-background/70 p-2 text-sm break-words whitespace-pre-wrap focus-visible:outline-2 focus-visible:outline-ring",
            !isText && "font-mono text-xs",
            clampable && !expanded && "line-clamp-3",
            clampable && expanded && "max-h-[28rem] overflow-auto",
          )}
        >
          {answer}
        </div>
      )}
      {thoughts.length > 0 && (
        <Disclosure label="Reasoning" count={thoughts.length}>
          {thoughts.map((thought, index) => (
            <p key={index} className="rounded-md bg-background/70 p-2 text-xs break-words whitespace-pre-wrap text-muted-foreground">
              {thought}
            </p>
          ))}
        </Disclosure>
      )}
      <div className="flex flex-wrap items-center gap-2">
        {clampable && (
          <Button type="button" variant="ghost" className="min-h-11" aria-expanded={expanded} aria-controls={bodyId} onClick={() => setExpanded((current) => !current)}>
            {expanded ? "Show less" : "Show more"}
          </Button>
        )}
        <Button type="button" variant="ghost" className="min-h-11" onClick={() => void copyText()}>
          {copy === "copied" ? <Check aria-hidden="true" /> : <Copy aria-hidden="true" />}
          Copy
        </Button>
        <span role="status" className="text-xs text-muted-foreground">
          {copy === "copied" ? "Copied." : copy === "failed" ? "Copy failed: the browser blocked clipboard access." : ""}
        </span>
      </div>
    </div>
  );
}
