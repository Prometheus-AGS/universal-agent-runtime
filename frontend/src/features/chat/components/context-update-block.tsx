import { DatabaseZapIcon } from "lucide-react";
import type { FC } from "react";

const STRATEGY_LABELS: Record<string, string> = {
  sliding_window: "Sliding window",
  progressive_summarization: "Progressive summarization",
  hierarchical_memory: "Hierarchical memory",
  keep_first_last: "Keep first + last",
  none: "No-op",
};

interface ContextUpdateBlockProps {
  strategy: string;
  messagesRemoved: number;
  tokensSaved: number;
  wasApplied: boolean;
  summaryGenerated: boolean;
}

export const ContextUpdateBlock: FC<ContextUpdateBlockProps> = ({ strategy, messagesRemoved, tokensSaved, summaryGenerated }) => {
  const strategyLabel = STRATEGY_LABELS[strategy] ?? strategy;
  const parts: string[] = [];
  if (messagesRemoved > 0) parts.push(`${messagesRemoved} message${messagesRemoved !== 1 ? "s" : ""} compacted`);
  if (tokensSaved > 0) parts.push(`~${tokensSaved.toLocaleString()} tokens freed`);
  if (summaryGenerated) parts.push("summary saved");

  return (
    <div className="my-1.5 flex items-center gap-2 rounded-xl bg-surface px-3 py-2">
      <DatabaseZapIcon size={11} className="shrink-0 text-fg-faint" />
      <span className="font-mono text-[10px] text-fg-sub">Context managed</span>
      <span className="font-mono text-[10px] text-fg-faint">·</span>
      <span className="font-mono text-[10px] text-fg-sub">{strategyLabel}</span>
      {parts.length > 0 && (
        <>
          <span className="font-mono text-[10px] text-fg-faint">·</span>
          <span className="font-mono text-[10px] text-fg-faint">{parts.join(" · ")}</span>
        </>
      )}
    </div>
  );
};
