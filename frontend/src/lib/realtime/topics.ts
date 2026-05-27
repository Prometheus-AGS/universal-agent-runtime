/**
 * Canonical mapping of UAR realtime topics → graph entity types.
 * Source of truth: `src/uar/realtime/mod.rs::EntityTopic` on the Rust side.
 */
import type { RealtimeAdapter } from "@prometheus-ags/prometheus-entity-management";
import { createUarSseAdapter } from "./uar-sse-adapter";

// EntityType names use CamelCase to match the schemas registered in
// `frontend/src/entities/schemas.ts`. The graph store keys events by these
// exact names; mixing cases would split the cache across two slots.
export const UAR_TOPICS = [
  { topic: "knowledge_bases", entityType: "KnowledgeBase" },
  { topic: "knowledge_documents", entityType: "Document" },
  { topic: "agents", entityType: "Agent" },
  { topic: "providers", entityType: "Provider" },
  { topic: "models", entityType: "Model" },
  { topic: "skills", entityType: "Skill" },
  { topic: "settings", entityType: "Setting" },
  // `threads` aliases the SurrealDB `sessions` table for the frontend.
  { topic: "threads", entityType: "Thread" },
  { topic: "memory", entityType: "Memory" },
  { topic: "compiler_sessions", entityType: "CompilerSession" },
] as const;

export function createAllUarAdapters(baseUrl = ""): RealtimeAdapter[] {
  return UAR_TOPICS.map(({ topic, entityType }) =>
    createUarSseAdapter({ topic, entityType, baseUrl }),
  );
}
