import type { Chunk } from "@/features/chat/model/chunk";
import { assertNever } from "@/shared/content";
import { A2uiChunkView } from "./a2ui-chunk";
import { ToolApprovalChunkView, ToolDeniedChunkView } from "./approval-chunk";
import { ArtifactChunkView } from "./artifact-chunk";
import { CitationChunkView } from "./citation-chunk";
import { ContextChunkView } from "./context-chunk";
import { MediaChunkView } from "./media-chunk";
import { MemoryChunkView } from "./memory-chunk";
import { DividerChunkView, ErrorChunkView, UsageChunkView } from "./meta-chunk";
import { ReasoningChunkView } from "./reasoning-chunk";
import { SkillChunkView } from "./skill-chunk";
import { TextChunkView } from "./text-chunk";
import { ToolChunkView } from "./tool-chunk";

export function ChunkRenderer({ chunk }: { chunk: Chunk }) {
  switch (chunk.kind) {
    case "text": case "markdown": return <TextChunkView chunk={chunk} />;
    case "reasoning": case "thinking": return <ReasoningChunkView chunk={chunk} />;
    case "tool-call": return <ToolChunkView chunk={chunk} />;
    case "tool-approval": return <ToolApprovalChunkView chunk={chunk} />;
    case "tool-denied": return <ToolDeniedChunkView chunk={chunk} />;
    case "skill-activation": return <SkillChunkView chunk={chunk} />;
    case "memory-recall": case "memory-mutation": case "memory-update": return <MemoryChunkView chunk={chunk} />;
    case "citation": case "rag-citations": return <CitationChunkView chunk={chunk} />;
    case "context-update": return <ContextChunkView chunk={chunk} />;
    case "a2ui-display": case "a2ui-input": return <A2uiChunkView chunk={chunk} />;
    case "artifact": return <ArtifactChunkView chunk={chunk} />;
    case "image": case "video": case "file": return <MediaChunkView chunk={chunk} />;
    case "divider": return <DividerChunkView chunk={chunk} />;
    case "usage": return <UsageChunkView chunk={chunk} />;
    case "error": return <ErrorChunkView chunk={chunk} />;
    case "state-snapshot": case "state-delta": case "step": case "raw": return null;
    default: return assertNever(chunk);
  }
}
