import { useRef } from "react";

import type {
  RunTraceCanonicalPhase,
  RunTraceSegment,
} from "@/features/chat/model/run-trace-types";
import { cn } from "@/lib/utils";

const PHASE_SURFACE: Record<RunTraceCanonicalPhase, string> = {
  context: "bg-[color-mix(in_srgb,var(--color-phase-context)_24%,var(--color-card))]",
  skill: "bg-[color-mix(in_srgb,var(--color-phase-skill)_24%,var(--color-card))]",
  memory: "bg-[color-mix(in_srgb,var(--color-phase-memory)_24%,var(--color-card))]",
  retrieval: "bg-[color-mix(in_srgb,var(--color-phase-retrieval)_24%,var(--color-card))]",
  reasoning: "bg-[color-mix(in_srgb,var(--color-phase-reasoning)_24%,var(--color-card))]",
  tool: "bg-[color-mix(in_srgb,var(--color-phase-tool)_24%,var(--color-card))]",
  generate: "bg-[color-mix(in_srgb,var(--color-phase-generate)_24%,var(--color-card))]",
};

function phaseLabel(segment: RunTraceSegment): string {
  return `${segment.phase} ${segment.durationMs.toFixed(0)} ms · ${segment.exactPercentage.toFixed(1)}%`;
}

export function RunTraceBar({
  segments,
  selectedPhase,
  onSelectPhase,
}: {
  segments: RunTraceSegment[];
  selectedPhase: RunTraceCanonicalPhase | null;
  onSelectPhase: (phase: RunTraceCanonicalPhase) => void;
}) {
  const items = useRef<Array<HTMLButtonElement | null>>([]);
  const activeIndex = Math.max(0, segments.findIndex((segment) => segment.phase === selectedPhase));

  const move = (index: number): void => {
    const next = Math.max(0, Math.min(index, segments.length - 1));
    const segment = segments[next];
    if (!segment) return;
    onSelectPhase(segment.phase);
    items.current[next]?.focus();
  };

  if (segments.length === 0) {
    return (
      <div className="rounded-xl bg-muted px-4 py-3 text-sm text-muted-foreground">
        Phase timing appears when a run reaches a terminal state.
      </div>
    );
  }

  return (
    <div
      role="listbox"
      aria-label="Run phases"
      aria-orientation="horizontal"
      className="flex min-h-12 w-full gap-1 rounded-xl bg-surface p-1"
    >
      {segments.map((segment, index) => (
        <button
          key={segment.phase}
          ref={(element) => { items.current[index] = element; }}
          type="button"
          role="option"
          aria-selected={segment.phase === selectedPhase}
          aria-label={phaseLabel(segment)}
          title={phaseLabel(segment)}
          tabIndex={index === activeIndex ? 0 : -1}
          className={cn(
            "min-h-11 min-w-12 overflow-hidden rounded-lg px-2 text-left font-mono text-[10px] font-semibold uppercase tracking-wide text-foreground transition-colors focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember",
            PHASE_SURFACE[segment.phase],
            segment.phase === selectedPhase && "bg-card-hov",
          )}
          style={{ flexGrow: segment.visualWeight, flexBasis: 0 }}
          onClick={() => onSelectPhase(segment.phase)}
          onKeyDown={(event) => {
            if (event.key === "ArrowRight") {
              event.preventDefault();
              move(index + 1);
            } else if (event.key === "ArrowLeft") {
              event.preventDefault();
              move(index - 1);
            } else if (event.key === "Home") {
              event.preventDefault();
              move(0);
            } else if (event.key === "End") {
              event.preventDefault();
              move(segments.length - 1);
            }
          }}
        >
          <span className="block truncate">{segment.phase}</span>
          <span className="block truncate text-fg-sub">
            {segment.durationMs.toFixed(0)} ms · {segment.exactPercentage.toFixed(1)}%
          </span>
        </button>
      ))}
    </div>
  );
}
