import type { A2uiDisplayChunk, ChunkBase } from "./chunk";

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown> : {};
}

/** The live display interpretation, shared with native journal recovery.
 * Declared intent reaches the existing renderer even for missing/unknown
 * profiles; content alone never grants A2UI capability or publication identity. */
export function artifactDisplayChunk(value: Record<string, unknown>, base: Omit<ChunkBase, "kind">): A2uiDisplayChunk | undefined {
  const metadata = record(value.metadata);
  const artifactType = typeof value.artifactType === "string" ? value.artifactType
    : typeof value.artifact_type === "string" ? value.artifact_type : "";
  const language = typeof value.language === "string" ? value.language : "";
  const isA2ui = artifactType === "a2ui" || artifactType === "display" || artifactType.startsWith("a2ui/")
    || ["a2ui", "application/a2ui+json", "application/vnd.uar.a2ui+json"].includes(language)
    || (typeof metadata.profile === "string" && metadata.profile.startsWith("uar.a2ui/"));
  if (!isA2ui) return undefined;
  const artifactId = typeof value.artifactId === "string" ? value.artifactId
    : typeof value.artifact_id === "string" ? value.artifact_id : undefined;
  const payload = { ...value, artifactType };
  return { ...base, kind: "a2ui-display", toolCallId: artifactId,
    profile: typeof metadata.profile === "string" ? metadata.profile : "",
    version: typeof metadata.protocol_version === "string" ? metadata.protocol_version : undefined,
    component: artifactType || "surface", payload,
    validation: value.validation === "invalid" || value.validation === "unknown-component" ? value.validation : "valid",
    validationError: typeof value.validationError === "string" ? value.validationError : undefined };
}
