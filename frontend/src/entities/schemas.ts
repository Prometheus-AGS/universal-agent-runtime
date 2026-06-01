import { registerSchema } from "@prometheus-ags/prometheus-entity-management";
import type { EntityId } from "@prometheus-ags/prometheus-entity-management";

const list = (type: string, filter: Record<string, EntityId>) => [type, filter];

export function registerAllSchemas() {
  registerSchema({
    type: "Provider",
    relations: {
      models: {
        cardinality: "hasMany",
        targetType: "Model",
        foreignKey: "provider_id",
        listKeyPrefix: (id) => list("Model", { provider_id: id }),
      },
      health: {
        cardinality: "hasMany",
        targetType: "RuntimeProviderHealth",
        foreignKey: "provider_id",
        listKeyPrefix: (id) => list("RuntimeProviderHealth", { provider_id: id }),
      },
    },
  });

  registerSchema({
    type: "Model",
    relations: {
      provider: {
        cardinality: "belongsTo",
        targetType: "Provider",
        foreignKey: "provider_id",
      },
    },
  });

  registerSchema({
    type: "Agent",
    relations: {
      sessions: {
        cardinality: "hasMany",
        targetType: "AgentSession",
        foreignKey: "agent_id",
        listKeyPrefix: (id) => list("AgentSession", { agent_id: id }),
      },
      runs: {
        cardinality: "hasMany",
        targetType: "RuntimeRun",
        foreignKey: "agent_id",
        listKeyPrefix: (id) => list("RuntimeRun", { agent_id: id }),
      },
    },
  });

  registerSchema({
    type: "AgentSession",
    relations: {
      agent: {
        cardinality: "belongsTo",
        targetType: "Agent",
        foreignKey: "agent_id",
      },
      thread: {
        cardinality: "belongsTo",
        targetType: "Thread",
        foreignKey: "session_id",
      },
    },
  });

  registerSchema({ type: "Skill" });
  registerSchema({ type: "Tool" });
  registerSchema({ type: "Setting" });
  registerSchema({ type: "SettingsType" });
  registerSchema({ type: "ProviderMeta" });

  registerSchema({
    type: "KnowledgeBase",
    relations: {
      documents: {
        cardinality: "hasMany",
        targetType: "Document",
        foreignKey: "kb_id",
        listKeyPrefix: (id) => list("Document", { kb_id: id }),
      },
    },
  });

  registerSchema({
    type: "Document",
    relations: {
      knowledgeBase: {
        cardinality: "belongsTo",
        targetType: "KnowledgeBase",
        foreignKey: "kb_id",
      },
    },
  });

  registerSchema({
    type: "Thread",
    relations: {
      sessions: {
        cardinality: "hasMany",
        targetType: "AgentSession",
        foreignKey: "session_id",
        listKeyPrefix: (id) => list("AgentSession", { session_id: id }),
      },
      runs: {
        cardinality: "hasMany",
        targetType: "RuntimeRun",
        foreignKey: "thread_id",
        listKeyPrefix: (id) => list("RuntimeRun", { thread_id: id }),
      },
    },
  });

  registerSchema({
    type: "RuntimeRun",
    relations: {
      agent: {
        cardinality: "belongsTo",
        targetType: "Agent",
        foreignKey: "agent_id",
      },
      thread: {
        cardinality: "belongsTo",
        targetType: "Thread",
        foreignKey: "thread_id",
      },
      steps: {
        cardinality: "hasMany",
        targetType: "RuntimeRunStep",
        foreignKey: "run_id",
        listKeyPrefix: (id) => list("RuntimeRunStep", { run_id: id }),
      },
      toolCalls: {
        cardinality: "hasMany",
        targetType: "RuntimeToolCall",
        foreignKey: "run_id",
        listKeyPrefix: (id) => list("RuntimeToolCall", { run_id: id }),
      },
      artifacts: {
        cardinality: "hasMany",
        targetType: "RuntimeArtifact",
        foreignKey: "run_id",
        listKeyPrefix: (id) => list("RuntimeArtifact", { run_id: id }),
      },
    },
  });

  registerSchema({
    type: "RuntimeRunStep",
    relations: {
      run: {
        cardinality: "belongsTo",
        targetType: "RuntimeRun",
        foreignKey: "run_id",
      },
    },
  });

  registerSchema({
    type: "RuntimeToolCall",
    relations: {
      run: {
        cardinality: "belongsTo",
        targetType: "RuntimeRun",
        foreignKey: "run_id",
      },
      step: {
        cardinality: "belongsTo",
        targetType: "RuntimeRunStep",
        foreignKey: "step_id",
      },
    },
  });

  registerSchema({ type: "RuntimeApproval" });
  registerSchema({ type: "RuntimeArtifact" });
  registerSchema({ type: "RuntimeMemoryEvent" });
  registerSchema({ type: "RuntimeAgUiEvent" });
  registerSchema({ type: "RuntimeA2uiSurface" });
  registerSchema({ type: "RuntimeModelRouteDecision" });
  registerSchema({ type: "RuntimeProviderHealth" });
}
