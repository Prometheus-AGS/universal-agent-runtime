import type { A2uiDisplayChunk, A2uiInputChunk } from "@/features/chat/model/chunk";
import { A2uiDisplayBlock, A2uiInputBlock } from "@/features/chat/components/a2ui-artifact-block";

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : {};
}

export function A2uiChunkView({ chunk }: { chunk: A2uiDisplayChunk | A2uiInputChunk }) {
  const payload = record(chunk.payload);
  if (chunk.kind === "a2ui-input") {
    return (
      <A2uiInputBlock
        runId={chunk.runId ?? chunk.requestId}
        artifactId={chunk.id}
        artifactType={chunk.component}
        title={typeof payload.title === "string" ? payload.title : "Input required"}
        content={typeof payload.content === "string" ? payload.content : JSON.stringify(payload)}
        metadata={record(payload.metadata)}
        status={chunk.status === "awaiting" ? "running" : chunk.status === "submitted" ? "complete" : "failed"}
        result={chunk.response === undefined ? undefined : JSON.stringify(chunk.response)}
      />
    );
  }
  return (
    <A2uiDisplayBlock
      artifactType={chunk.component}
      title={typeof payload.title === "string" ? payload.title : "Generated surface"}
      content={typeof payload.content === "string" ? payload.content : JSON.stringify(payload)}
      language={typeof payload.language === "string" ? payload.language : undefined}
      profile={chunk.profile}
      validation={chunk.validation}
      validationError={chunk.validationError}
    />
  );
}
