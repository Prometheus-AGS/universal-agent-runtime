import { SparklesIcon } from "lucide-react";
import type { SkillActivationChunk } from "@/features/chat/model/chunk";
import { ChunkMeta, ChunkSurface } from "./chunk-surface";

export function SkillChunkView({ chunk }: { chunk: SkillActivationChunk }) {
  return (
    <ChunkSurface label="Skill activation" live={chunk.status === "active"}>
      <div className="flex items-center gap-2">
        <SparklesIcon size={15} className="text-warning" aria-hidden="true" />
        <span className="font-mono text-xs font-medium">{chunk.skillName}</span>
        <span className="ml-auto text-xs">{chunk.status}</span>
      </div>
      {chunk.selectionMethod ? <div className="mt-1"><ChunkMeta>{chunk.selectionMethod}{chunk.score !== undefined ? ` · score ${chunk.score}` : ""}{chunk.threshold !== undefined ? ` / threshold ${chunk.threshold}` : ""}</ChunkMeta></div> : null}
    </ChunkSurface>
  );
}
