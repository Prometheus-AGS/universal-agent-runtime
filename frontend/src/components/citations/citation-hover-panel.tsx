import type { FC } from "react";
import { BookTextIcon } from "lucide-react";
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@/components/ui/hover-card";
import type { RagCitationMarker } from "@/types/chat-content";
import { useMessageCitations } from "@/hooks/use-message-citations";

/**
 * A single `[n]` citation marker badge. On hover, shows the source chunk's
 * document name, relevance score, and snippet — the "hover-to-source panel"
 * for RAG citations (Change 13, `rag-citation-stream`).
 *
 * Pure presentational component: receives its data as a prop, no store or
 * fetch access (Components -> Hooks -> Stores -> Services layering).
 */
export const CitationBadge: FC<{ citation: RagCitationMarker }> = ({ citation }) => (
  <HoverCard>
    <HoverCardTrigger
      render={
        <button
          type="button"
          className="inline-flex size-4 items-center justify-center rounded-full bg-muted align-super font-mono text-[10px] font-medium text-muted-foreground hover:bg-primary hover:text-primary-foreground"
        />
      }
    >
      {citation.marker}
      <span className="sr-only"> — source: {citation.documentName}</span>
    </HoverCardTrigger>
    <HoverCardContent className="w-72 space-y-1.5" side="top">
      <div className="flex items-center gap-1.5 text-xs font-medium text-foreground">
        <BookTextIcon className="size-3.5 shrink-0" />
        <span className="truncate">{citation.documentName}</span>
      </div>
      <p className="line-clamp-4 text-xs text-muted-foreground">{citation.snippet}</p>
      <div className="flex items-center justify-between pt-1 text-[10px] text-muted-foreground">
        <span>Relevance {(citation.relevanceScore * 100).toFixed(0)}%</span>
        <span className="font-mono">chunk:{citation.chunkId.slice(0, 8)}</span>
      </div>
    </HoverCardContent>
  </HoverCard>
);

/**
 * Ordered strip of citation badges for one message's RAG citation stream.
 * Renders nothing when there are no citations. Reads its data through
 * `useMessageCitations` (the hooks layer) rather than a store directly.
 */
export const MessageCitations: FC<{ threadId: string | null; messageId: string }> = ({
  threadId,
  messageId,
}) => {
  const citations = useMessageCitations(threadId, messageId);
  if (citations.length === 0) return null;

  return (
    <div className="ml-1 flex flex-wrap items-center gap-1 pt-1" aria-label="Sources">
      <span className="text-[10px] text-muted-foreground">Sources:</span>
      {citations.map((c) => (
        <CitationBadge key={`${c.chunkId}-${c.marker}`} citation={c} />
      ))}
    </div>
  );
};
