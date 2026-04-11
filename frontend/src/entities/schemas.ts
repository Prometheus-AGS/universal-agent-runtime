import { registerSchema } from "@prometheus-ags/prometheus-entity-management";

export function registerAllSchemas() {
  registerSchema({
    type: "Provider",
    idField: "id",
    relations: [
      { type: "Model", field: "provider_id", kind: "hasMany" },
    ],
  });

  registerSchema({
    type: "Model",
    idField: "id",
    relations: [
      { type: "Provider", field: "provider_id", kind: "belongsTo" },
    ],
  });

  registerSchema({
    type: "Agent",
    idField: "id",
    relations: [
      { type: "Skill", field: "skills", kind: "hasMany" },
      { type: "Tool", field: "tools", kind: "hasMany" },
      { type: "KnowledgeBase", field: "knowledge_bases", kind: "hasMany" },
    ],
  });

  registerSchema({
    type: "AgentSession",
    idField: "id",
    relations: [
      { type: "Agent", field: "agent_id", kind: "belongsTo" },
      { type: "Thread", field: "session_id", kind: "belongsTo" },
    ],
  });

  registerSchema({ type: "Skill", idField: "id" });
  registerSchema({ type: "Tool", idField: "id" });

  registerSchema({
    type: "KnowledgeBase",
    idField: "id",
    relations: [
      { type: "Document", field: "kb_id", kind: "hasMany" },
    ],
  });

  registerSchema({
    type: "Document",
    idField: "id",
    relations: [
      { type: "KnowledgeBase", field: "kb_id", kind: "belongsTo" },
    ],
  });

  registerSchema({
    type: "Thread",
    idField: "id",
    relations: [
      { type: "AgentSession", field: "session_id", kind: "hasMany" },
    ],
  });
}
