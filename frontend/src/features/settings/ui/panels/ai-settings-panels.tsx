import { useCallback, useState } from "react";
import { Loader2 } from "lucide-react";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { useSettings } from "../../model/use-settings";
import { NamespacePanel } from "../generic-schema-panel";
import {
  AdvancedSection,
  ErrorBanner,
  Field,
  MaskedInput,
  PanelHeader,
  SavedBanner,
  SettingSelect,
  Toggle,
} from "../settings-primitives";

export function ProviderPanel() {
  const {
    values,
    settings,
    loading,
    saving,
    error,
    setSetting,
    saveAll,
    reload,
  } = useSettings("provider");
  const [savedFlash, setSavedFlash] = useState(false);

  const providerEntries = Object.values(settings).sort((a, b) =>
    a.key.localeCompare(b.key),
  );

  const handleSave = useCallback(async () => {
    await saveAll();
    setSavedFlash(true);
    setTimeout(() => setSavedFlash(false), 2500);
  }, [saveAll]);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <PanelHeader
        title="LLM Providers"
        subtitle={`${providerEntries.length} provider(s) configured`}
        saving={saving}
        loading={loading}
        onSave={() => void handleSave()}
        onReload={() => void reload()}
      />
      <div className="flex-1 overflow-y-auto px-6 py-5 space-y-4">
        {loading && (
          <div className="flex items-center gap-2">
            <Loader2 size={15} className="animate-spin text-muted-foreground" />
            <span className="font-mono text-xs text-muted-foreground">
              Loading…
            </span>
          </div>
        )}
        <ErrorBanner error={error} />
        <SavedBanner show={savedFlash} />
        {!loading && providerEntries.length === 0 && (
          <p className="font-mono text-xs text-muted-foreground">
            No providers configured. Add providers in your config file to manage
            them here.
          </p>
        )}
        {providerEntries.map((s) => {
          const data = (values[s.key] ?? {}) as Record<string, unknown>;
          const leafKey = s.key; // full key like provider.openai
          const setField = (field: string, value: unknown) => {
            const updated = { ...data, [field]: value };
            setSetting(leafKey, updated);
          };
          const enabled = data.enabled !== false;
          return (
            <div
              key={s.key}
              className={cn(
                "rounded-xl border p-4 space-y-4",
                enabled
                  ? "border-border bg-card"
                  : "border-border/50 bg-muted/30 opacity-60",
              )}
            >
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-display text-sm font-semibold text-foreground">
                    {(data.display_name as string) ?? s.name}
                  </p>
                  <p className="font-mono text-xs text-muted-foreground">
                    {s.key}
                  </p>
                </div>
                <Toggle
                  value={enabled}
                  onChange={(v) => setField("enabled", v)}
                />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <Field label="Base URL">
                  <Input
                    value={(data.base_url as string) ?? ""}
                    onChange={(e) => setField("base_url", e.target.value)}
                    placeholder="https://api.example.com/v1"
                    className="font-mono text-xs"
                  />
                </Field>
                <Field label="Protocol">
                  <SettingSelect
                    value={(data.protocol as string) ?? "auto"}
                    options={[
                      { value: "auto", label: "Auto" },
                      { value: "chat", label: "Chat Completions" },
                      { value: "responses", label: "Responses API" },
                    ]}
                    onChange={(v) => setField("protocol", v)}
                  />
                </Field>
                <Field label="API Key" hint="Masked for security">
                  <MaskedInput
                    value={(data.api_key as string) ?? ""}
                    onChange={(v) => setField("api_key", v)}
                    placeholder="sk-..."
                  />
                </Field>
                <Field label="Default Model">
                  <Input
                    value={(data.default_model as string) ?? ""}
                    onChange={(e) => setField("default_model", e.target.value)}
                    placeholder="gpt-5.2"
                    className="font-mono text-xs"
                  />
                </Field>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

// --- Vision -----------------------------------------------------------------

export function VisionPanel() {
  return (
    <NamespacePanel namespace="vision" title="Vision Configuration">
      {({ val, set }) => (
        <>
          <Field
            label="Vision Model Override"
            hint="Leave blank to use the active model's capabilities"
          >
            <Input
              value={(val("model") as string) ?? ""}
              onChange={(e) => set("model", e.target.value || null)}
              placeholder="gpt-5.2 (inherits active model)"
              className="font-mono text-xs"
            />
          </Field>
          <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
            <div>
              <p className="font-mono text-xs font-medium text-foreground">
                Auto-detect Vision Capability
              </p>
              <p className="font-mono text-xs text-muted-foreground">
                Automatically probe model capabilities on first use
              </p>
            </div>
            <Toggle
              value={(val("auto_detect") as boolean) ?? true}
              onChange={(v) => set("auto_detect", v)}
            />
          </div>
        </>
      )}
    </NamespacePanel>
  );
}

// --- Context Management -----------------------------------------------------

const CONTEXT_STRATEGIES = [
  { value: "sliding_window", label: "Sliding Window" },
  { value: "keep_first_last", label: "Keep First & Last" },
  { value: "progressive_summarization", label: "Progressive Summarization" },
  { value: "hierarchical_memory", label: "Hierarchical Memory" },
  { value: "none", label: "None (no management)" },
];

export function ContextManagementPanel() {
  return (
    <NamespacePanel
      namespace="context_management"
      title="Context Management"
      subtitle="Global defaults — agents can override individually"
      hint="Context management controls how conversations handle token limits. The strategy determines what happens when a conversation gets too long — whether to summarize, trim older messages, or use a sliding window. Most users can leave the defaults."
    >
      {({ val, set }) => (
        <>
          <Field
            label="Strategy"
            hint="How to handle conversations that approach the token limit"
            defaultValue="sliding_window"
          >
            <SettingSelect
              value={(val("strategy") as string) ?? "sliding_window"}
              options={CONTEXT_STRATEGIES}
              onChange={(v) => set("strategy", v)}
            />
          </Field>
          <Field
            label="Trigger Threshold"
            hint="Start managing context when usage exceeds this ratio"
            defaultValue="0.85"
          >
            <Input
              type="number"
              step={0.05}
              min={0.1}
              max={1.0}
              value={(val("trigger_threshold") as number) ?? 0.85}
              onChange={(e) =>
                set("trigger_threshold", parseFloat(e.target.value))
              }
              className="font-mono text-xs"
            />
          </Field>
          <AdvancedSection label="Token limits, summarization & model override">
            <div className="grid grid-cols-2 gap-4">
              <Field
                label="Max Tokens"
                hint="Leave blank to use model's context window"
              >
                <Input
                  type="number"
                  value={(val("max_tokens") as number) ?? ""}
                  onChange={(e) =>
                    set(
                      "max_tokens",
                      e.target.value ? parseInt(e.target.value) : null,
                    )
                  }
                  placeholder="Blank = model limit"
                  min={512}
                  className="font-mono text-xs"
                />
              </Field>
              <Field
                label="Max Messages"
                hint="Sliding window message limit"
                defaultValue="unlimited"
              >
                <Input
                  type="number"
                  value={(val("max_messages") as number) ?? ""}
                  onChange={(e) =>
                    set(
                      "max_messages",
                      e.target.value ? parseInt(e.target.value) : null,
                    )
                  }
                  placeholder="Unlimited"
                  min={1}
                  className="font-mono text-xs"
                />
              </Field>
            </div>
            <Field
              label="Summary Budget (tokens)"
              hint="Token budget for generated summaries"
              defaultValue="1000"
            >
              <Input
                type="number"
                value={(val("summary_budget") as number) ?? 1000}
                onChange={(e) =>
                  set("summary_budget", parseInt(e.target.value))
                }
                min={100}
                className="font-mono text-xs"
              />
            </Field>
            <Field
              label="Summarization Model"
              hint="Override which model performs summarization"
              defaultValue="active model"
            >
              <Input
                value={(val("summarization_model") as string) ?? ""}
                onChange={(e) =>
                  set("summarization_model", e.target.value || null)
                }
                placeholder="Inherits active model"
                className="font-mono text-xs"
              />
            </Field>
          </AdvancedSection>
        </>
      )}
    </NamespacePanel>
  );
}

// --- RAG & Chunking ---------------------------------------------------------

const CHUNKING_STRATEGIES = [
  { value: "fixed_size", label: "Fixed Size (chars)" },
  { value: "token", label: "Token-based" },
  { value: "recursive", label: "Recursive (semantic boundaries)" },
  { value: "sentence", label: "Sentence" },
  { value: "document", label: "Document (no chunking)" },
  { value: "semantic", label: "Semantic (embedding similarity)" },
  { value: "agentic", label: "Agentic (LLM-guided)" },
];

export function RagPanel() {
  return (
    <NamespacePanel
      namespace="rag"
      title="RAG & Chunking"
      subtitle="Retrieval-augmented generation and document chunking defaults"
      hint="These settings control how uploaded documents are split into searchable pieces (chunks) and how similarity search works. The chunking strategy affects search quality — 'recursive' works well for most documents. Only change these if search results aren't meeting your needs."
    >
      {({ val, set }) => {
        const strategy = (val("chunking_strategy") as string) ?? "recursive";
        return (
          <>
            <Field label="Chunking Strategy">
              <SettingSelect
                value={strategy}
                options={CHUNKING_STRATEGIES}
                onChange={(v) => set("chunking_strategy", v)}
              />
            </Field>
            <div className="grid grid-cols-2 gap-4">
              {(strategy === "fixed_size" || strategy === "recursive") && (
                <Field label="Chunk Size (chars)">
                  <Input
                    type="number"
                    value={(val("chunk_size") as number) ?? 1024}
                    onChange={(e) =>
                      set("chunk_size", parseInt(e.target.value))
                    }
                    min={64}
                    className="font-mono text-xs"
                  />
                </Field>
              )}
              {strategy === "token" && (
                <Field label="Chunk Size (tokens)">
                  <Input
                    type="number"
                    value={(val("chunk_tokens") as number) ?? 256}
                    onChange={(e) =>
                      set("chunk_tokens", parseInt(e.target.value))
                    }
                    min={16}
                    className="font-mono text-xs"
                  />
                </Field>
              )}
              {strategy === "semantic" && (
                <Field
                  label="Semantic Merge Threshold"
                  hint="0.0 – 1.0 (higher = merge less)"
                >
                  <Input
                    type="number"
                    step={0.05}
                    min={0.0}
                    max={1.0}
                    value={(val("semantic_threshold") as number) ?? 0.75}
                    onChange={(e) =>
                      set("semantic_threshold", parseFloat(e.target.value))
                    }
                    className="font-mono text-xs"
                  />
                </Field>
              )}
            </div>
            <div className="rounded-lg border border-border bg-card p-4 space-y-3">
              <p className="font-mono text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                Embedding
              </p>
              <div className="grid grid-cols-2 gap-4">
                <Field label="Embedding Provider ID" defaultValue="openai">
                  <Input
                    value={(val("embedding_provider") as string) ?? ""}
                    onChange={(e) => set("embedding_provider", e.target.value)}
                    placeholder="openai"
                    className="font-mono text-xs"
                  />
                </Field>
                <Field
                  label="Embedding Model ID"
                  defaultValue="text-embedding-3-small"
                >
                  <Input
                    value={(val("embedding_model") as string) ?? ""}
                    onChange={(e) => set("embedding_model", e.target.value)}
                    placeholder="text-embedding-3-small"
                    className="font-mono text-xs"
                  />
                </Field>
              </div>
            </div>
          </>
        );
      }}
    </NamespacePanel>
  );
}

// --- Knowledge Bases --------------------------------------------------------

export function KnowledgeBasesPanel() {
  return (
    <NamespacePanel
      namespace="knowledge_bases"
      title="Knowledge Bases"
      subtitle="Default embedding and chunking configuration for all knowledge bases"
    >
      {({ val, set }) => {
        const defaultKb = (val("default") as Record<string, unknown>) ?? {};
        const setDefault = (field: string, value: unknown) =>
          set("default", { ...defaultKb, [field]: value });
        return (
          <>
            <div className="grid grid-cols-2 gap-4">
              <Field label="Default Embedding Provider">
                <Input
                  value={(defaultKb.embedding_provider as string) ?? ""}
                  onChange={(e) =>
                    setDefault("embedding_provider", e.target.value)
                  }
                  placeholder="openai"
                  className="font-mono text-xs"
                />
              </Field>
              <Field label="Default Embedding Model">
                <Input
                  value={(defaultKb.embedding_model as string) ?? ""}
                  onChange={(e) =>
                    setDefault("embedding_model", e.target.value)
                  }
                  placeholder="text-embedding-3-small"
                  className="font-mono text-xs"
                />
              </Field>
              <Field label="Chunking Strategy">
                <Input
                  value={(defaultKb.chunking_strategy as string) ?? ""}
                  onChange={(e) =>
                    setDefault("chunking_strategy", e.target.value)
                  }
                  placeholder="recursive"
                  className="font-mono text-xs"
                />
              </Field>
              <Field label="Chunk Size">
                <Input
                  type="number"
                  value={(defaultKb.chunk_size as number) ?? ""}
                  onChange={(e) =>
                    setDefault(
                      "chunk_size",
                      e.target.value ? parseInt(e.target.value) : null,
                    )
                  }
                  placeholder="1024"
                  className="font-mono text-xs"
                />
              </Field>
            </div>
          </>
        );
      }}
    </NamespacePanel>
  );
}

