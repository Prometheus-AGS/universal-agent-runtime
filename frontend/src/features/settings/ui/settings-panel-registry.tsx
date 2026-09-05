import type { ReactNode } from "react";
import { GenericSchemaPanel } from "./generic-schema-panel";
import {
  ContextManagementPanel,
  KnowledgeBasesPanel,
  ProviderPanel,
  RagPanel,
  VisionPanel,
} from "./panels/ai-settings-panels";
import {
  PromptCachingPanel,
  UserSettingsPanel,
} from "./panels/caching-user-settings-panels";
import {
  FileProcessingPanel,
  KreuzbergPanel,
  MistralOcrPanel,
  UnstructuredPanel,
} from "./panels/file-processing-settings-panels";
import {
  AgentConfigPanel,
  GovernancePanel,
  IntentClassifierPanel,
  SkillConfigPanel,
} from "./panels/governance-settings-panels";
import { MemoryPanel } from "./panels/memory-settings-panel";
import { ResiliencePanel } from "./panels/resilience-settings-panel";
import { GlobalPresentationAssignment } from "@/features/presentations";

export const PANEL_MAP: Record<string, () => ReactNode> = {
  presentation_policy: () => <div className="min-h-0 flex-1 overflow-y-auto p-4 sm:p-6"><div className="max-w-3xl"><GlobalPresentationAssignment /></div></div>,
  llm: () => (
    <GenericSchemaPanel
      namespace="llm"
      title="LLM Configuration"
      subtitle="Global model defaults, protocol, timeout, cache, and budget settings"
    />
  ),
  provider: () => <ProviderPanel />,
  vision: () => <VisionPanel />,
  context_management: () => <ContextManagementPanel />,
  context_strategy: () => (
    <GenericSchemaPanel
      namespace="context_strategy"
      title="Context Strategy"
      subtitle="Runtime conversation trimming and strategy configuration"
    />
  ),
  rag: () => <RagPanel />,
  knowledge_bases: () => <KnowledgeBasesPanel />,
  memory: () => <MemoryPanel />,
  models: () => (
    <GenericSchemaPanel
      namespace="models"
      title="Model Files"
      subtitle="Tokenizer and embedding model file locations"
    />
  ),
  file_processing: () => <FileProcessingPanel />,
  unstructured: () => <UnstructuredPanel />,
  mistral_ocr: () => <MistralOcrPanel />,
  kreuzberg: () => <KreuzbergPanel />,
  resilience: () => <ResiliencePanel />,
  server: () => (
    <GenericSchemaPanel
      namespace="server"
      title="Server"
      subtitle="HTTP, gRPC, logging, and shutdown settings"
    />
  ),
  persistence: () => (
    <GenericSchemaPanel
      namespace="persistence"
      title="Persistence"
      subtitle="Database, vector, cache, and SurrealDB connection settings"
    />
  ),
  sandbox: () => (
    <GenericSchemaPanel
      namespace="sandbox"
      title="Sandbox"
      subtitle="Code execution runtime and remote sandbox settings"
    />
  ),
  intent_classifier: () => <IntentClassifierPanel />,
  security: () => (
    <GenericSchemaPanel
      namespace="security"
      title="Security"
      subtitle="JWT, admin mutation, and authentication settings"
    />
  ),
  governance: () => <GovernancePanel />,
  sycophancy: () => (
    <GenericSchemaPanel
      namespace="sycophancy"
      title="Sycophancy Detection"
      subtitle="LLM response quality guardrail settings"
    />
  ),
  agent_config: () => <AgentConfigPanel />,
  skill_config: () => <SkillConfigPanel />,
  native_tools: () => (
    <GenericSchemaPanel
      namespace="native_tools"
      title="Native Tools"
      subtitle="File, web, terminal, and session tool controls"
    />
  ),
  skill_evolution: () => (
    <GenericSchemaPanel
      namespace="skill_evolution"
      title="Skill Evolution"
      subtitle="Automatic skill reflection, mutation, and validation settings"
    />
  ),
  acp: () => (
    <GenericSchemaPanel
      namespace="acp"
      title="ACP Server"
      subtitle="Agent Communication Protocol runtime settings"
    />
  ),
  llm_failover: () => (
    <GenericSchemaPanel
      namespace="llm_failover"
      title="LLM Failover"
      subtitle="Fallback models, provider cooldowns, and failover behavior"
    />
  ),
  prompt_caching: () => <PromptCachingPanel />,
  user_settings: () => <UserSettingsPanel />,
};
