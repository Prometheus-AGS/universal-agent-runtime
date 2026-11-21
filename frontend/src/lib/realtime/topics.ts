/**
 * Canonical mapping of UAR realtime topics → graph entity types.
 * Source of truth: `src/uar/realtime/mod.rs::EntityTopic` on the Rust side.
 */
import {
  projectCatalogModel,
  recordModelCatalogSnapshotAccepted,
  recordModelCatalogSnapshotRejected,
} from "@/platform/entities";
import type { EntityChange, RealtimeAdapter } from "@/platform/entities";
import {
  createUarSseAdapter,
  type UarSnapshotOutcome,
  type UarSnapshotRecordProjection,
} from "./uar-sse-adapter";

interface UarTopicDefinition {
  readonly topic: string;
  readonly entityType: string;
  /** Optional projection applied to each snapshot record; a throw rejects the whole snapshot. */
  readonly projectRecord?: UarSnapshotRecordProjection;
  /** Optional observer of snapshot ingestion results, used to surface a rejection. */
  readonly onSnapshotOutcome?: (outcome: UarSnapshotOutcome) => void;
}

function recordCatalogSnapshotOutcome(outcome: UarSnapshotOutcome): void {
  if (outcome.ok) recordModelCatalogSnapshotAccepted();
  else recordModelCatalogSnapshotRejected(outcome.error);
}

// EntityType names use CamelCase to match the schemas registered in
// `frontend/src/entities/schemas.ts`. The graph store keys events by these
// exact names; mixing cases would split the cache across two slots.
export const UAR_TOPICS: readonly UarTopicDefinition[] = [
  { topic: "knowledge_bases", entityType: "KnowledgeBase" },
  { topic: "knowledge_documents", entityType: "Document" },
  { topic: "agents", entityType: "Agent" },
  { topic: "providers", entityType: "Provider" },
  {
    topic: "models",
    entityType: "Model",
    projectRecord: (record) => projectCatalogModel(record),
    onSnapshotOutcome: recordCatalogSnapshotOutcome,
  },
  { topic: "skills", entityType: "Skill" },
  { topic: "settings", entityType: "Setting" },
  // `threads` aliases the SurrealDB `sessions` table for the frontend.
  { topic: "threads", entityType: "Thread" },
  { topic: "memory", entityType: "Memory" },
  { topic: "compiler_sessions", entityType: "CompilerSession" },
];

export function createAllUarAdapters(
  baseUrl = "",
  onSettingChange?: (change: EntityChange) => void,
): RealtimeAdapter[] {
  return UAR_TOPICS.map(({ topic, entityType, projectRecord, onSnapshotOutcome }) =>
    createUarSseAdapter({
      topic,
      entityType,
      baseUrl,
      projectRecord,
      onSnapshotOutcome,
      onChange: entityType === "Setting" ? onSettingChange : undefined,
    }),
  );
}
