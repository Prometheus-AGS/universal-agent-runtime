import type { FC } from "react";
import {
  Bot,
  Brain,
  Database,
  Eye,
  FileText,
  Layers,
  Scissors,
  Server,
  ShieldCheck,
  SlidersHorizontal,
  User,
  Zap,
} from "lucide-react";

export type NavCategory =
  | "AI & LLM"
  | "File Processing"
  | "Infrastructure"
  | "Governance & Agents"
  | "Caching & Users";

export interface NavItem {
  key: string;
  label: string;
  subtitle: string;
  icon: FC<{ size?: number; className?: string }>;
  category: NavCategory;
}

export const NAV_ITEMS: NavItem[] = [
  {
    key: "llm",
    label: "LLM Configuration",
    subtitle:
      "Global defaults for LLM model, protocol, timeouts, and cost tracking",
    icon: Server,
    category: "AI & LLM",
  },
  {
    key: "provider",
    label: "Provider Overrides",
    subtitle: "Per-provider protocol & keys",
    icon: Server,
    category: "AI & LLM",
  },
  {
    key: "vision",
    label: "Vision",
    subtitle: "Image analysis settings",
    icon: Eye,
    category: "AI & LLM",
  },
  {
    key: "context_management",
    label: "Context Management",
    subtitle: "Token budgets & strategies",
    icon: Layers,
    category: "AI & LLM",
  },
  {
    key: "context_strategy",
    label: "Context Strategy",
    subtitle: "Runtime conversation trimming",
    icon: Layers,
    category: "AI & LLM",
  },
  {
    key: "rag",
    label: "RAG & Chunking",
    subtitle: "Embedding & chunk strategies",
    icon: Scissors,
    category: "AI & LLM",
  },
  {
    key: "knowledge_bases",
    label: "Knowledge Bases",
    subtitle: "Default KB settings",
    icon: Database,
    category: "AI & LLM",
  },
  {
    key: "memory",
    label: "Memory",
    subtitle: "Agent memory behavior",
    icon: Brain,
    category: "AI & LLM",
  },
  {
    key: "models",
    label: "Model Files",
    subtitle: "Tokenizer and embedding model paths",
    icon: Database,
    category: "AI & LLM",
  },
  {
    key: "file_processing",
    label: "File Processing",
    subtitle: "Upload limits & providers",
    icon: FileText,
    category: "File Processing",
  },
  {
    key: "unstructured",
    label: "Unstructured API",
    subtitle: "Unstructured.io integration",
    icon: FileText,
    category: "File Processing",
  },
  {
    key: "mistral_ocr",
    label: "Mistral OCR",
    subtitle: "Mistral document extraction",
    icon: SlidersHorizontal,
    category: "File Processing",
  },
  {
    key: "kreuzberg",
    label: "Kreuzberg OCR",
    subtitle: "Local OCR engine",
    icon: SlidersHorizontal,
    category: "File Processing",
  },
  {
    key: "resilience",
    label: "Resilience",
    subtitle: "Rate limits, retries, timeouts",
    icon: Brain,
    category: "Infrastructure",
  },
  {
    key: "server",
    label: "Server",
    subtitle: "HTTP and gRPC listener settings",
    icon: Server,
    category: "Infrastructure",
  },
  {
    key: "persistence",
    label: "Persistence",
    subtitle: "Database and vector storage",
    icon: Database,
    category: "Infrastructure",
  },
  {
    key: "sandbox",
    label: "Sandbox",
    subtitle: "Code execution runtime",
    icon: SlidersHorizontal,
    category: "Infrastructure",
  },
  {
    key: "intent_classifier",
    label: "Intent Classifier",
    subtitle: "Request routing rules",
    icon: Brain,
    category: "Infrastructure",
  },
  {
    key: "security",
    label: "Security",
    subtitle: "JWT and settings mutation controls",
    icon: ShieldCheck,
    category: "Governance & Agents",
  },
  {
    key: "governance",
    label: "Governance",
    subtitle: "Policies & guardrails",
    icon: ShieldCheck,
    category: "Governance & Agents",
  },
  {
    key: "sycophancy",
    label: "Sycophancy Detection",
    subtitle: "LLM response quality guardrail",
    icon: ShieldCheck,
    category: "Governance & Agents",
  },
  {
    key: "agent_config",
    label: "Agents",
    subtitle: "Default agent behavior",
    icon: Bot,
    category: "Governance & Agents",
  },
  {
    key: "skill_config",
    label: "Skills",
    subtitle: "Skill activation defaults",
    icon: Zap,
    category: "Governance & Agents",
  },
  {
    key: "native_tools",
    label: "Native Tools",
    subtitle: "File, web, terminal, and session tools",
    icon: SlidersHorizontal,
    category: "Governance & Agents",
  },
  {
    key: "skill_evolution",
    label: "Skill Evolution",
    subtitle: "Automatic skill reflection",
    icon: Zap,
    category: "Governance & Agents",
  },
  {
    key: "acp",
    label: "ACP Server",
    subtitle: "Agent Communication Protocol",
    icon: Server,
    category: "Governance & Agents",
  },
  {
    key: "llm_failover",
    label: "LLM Failover",
    subtitle: "Fallback models and provider cooldowns",
    icon: Server,
    category: "AI & LLM",
  },
  {
    key: "prompt_caching",
    label: "Prompt Caching",
    subtitle: "Cache scope & TTL",
    icon: Zap,
    category: "Caching & Users",
  },
  {
    key: "user_settings",
    label: "User Settings",
    subtitle: "Per-user preferences",
    icon: User,
    category: "Caching & Users",
  },
];

export const CATEGORIES: NavCategory[] = [
  "AI & LLM",
  "File Processing",
  "Infrastructure",
  "Governance & Agents",
  "Caching & Users",
];

