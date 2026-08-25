import { useCallback, useEffect, useRef, useState } from "react";
import { Loader2 } from "lucide-react";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { useSettings } from "../../model/use-settings";
import { NamespacePanel } from "../generic-schema-panel";
import { getProviderModelOptions } from "./provider-model-options";
import {
  AdvancedSection,
  ErrorBanner,
  Field,
  MaskedInput,
  PanelHeader,
  SavedBanner,
  SettingModelPicker,
  SettingSelect,
  Toggle,
} from "../settings-primitives";

function providerControlId(providerKey: string, field: string) {
  return `provider-${encodeURIComponent(providerKey)}-${field}`;
}

export function ProviderPanel() {
  const {
    values,
    settings,
    dirty,
    loading,
    refreshing,
    saving,
    error,
    setSetting,
    saveAll,
    reload,
  } = useSettings("provider");
  const [savedFlash, setSavedFlash] = useState(false);
  const savedTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hasDirty = Object.keys(dirty).length > 0;
  const reloadHint = hasDirty ? "Save changes before refreshing." : saving ? "Saving changes…" : refreshing ? "Refreshing settings…" : undefined;

  const providerEntries = Object.values(settings).sort((a, b) =>
    a.key.localeCompare(b.key),
  );

  const handleSave = useCallback(async () => {
    try {
      await saveAll();
    } catch {
      // The settings store keeps the actionable error and pending drafts.
      return;
    }
    setSavedFlash(true);
    if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
    savedTimerRef.current = setTimeout(() => setSavedFlash(false), 2500);
  }, [saveAll]);

  useEffect(
    () => () => {
      if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
    },
    [],
  );

  useEffect(() => {
    if (!hasDirty) return;
    const preventUnsavedUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", preventUnsavedUnload);
    return () =>
      window.removeEventListener("beforeunload", preventUnsavedUnload);
  }, [hasDirty]);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <PanelHeader
        title="LLM Providers"
        subtitle={`${providerEntries.length} configured provider${providerEntries.length === 1 ? "" : "s"}`}
        saving={saving}
        loading={refreshing}
        saveDisabled={!hasDirty}
        reloadDisabled={hasDirty || saving}
        statusText={hasDirty ? "Unsaved changes" : undefined}
        reloadHint={reloadHint}
        onSave={() => void handleSave()}
        onReload={() => void reload()}
      />
      <div className="flex-1 overflow-y-auto px-6 py-5 space-y-4">
        {loading && (
          <div
            role="status"
            aria-live="polite"
            className="flex items-center gap-2"
          >
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
          const displayName = (data.display_name as string) ?? s.name;
          const modelOptions = getProviderModelOptions(data);
          const currentModel = (data.default_model as string) ?? "";
          const currentModelIsAvailable = modelOptions.some(
            (option) => option.value === currentModel,
          );
          const modelHint =
            modelOptions.length === 0
              ? "No enabled models are available for this provider."
              : currentModel && !currentModelIsAvailable
                ? "Current model is unavailable. Choose an enabled model."
                : undefined;
          const leafKey = s.key; // full key like provider.openai
          const providerHeadingId = providerControlId(s.key, "heading");
          const baseUrlId = providerControlId(s.key, "base-url");
          const protocolId = providerControlId(s.key, "protocol");
          const apiKeyId = providerControlId(s.key, "api-key");
          const apiKeyHintId = providerControlId(s.key, "api-key-hint");
          const defaultModelId = providerControlId(s.key, "default-model");
          const defaultModelHintId = providerControlId(
            s.key,
            "default-model-hint",
          );
          const enabledId = providerControlId(s.key, "enabled");
          const providerIsDirty = Object.prototype.hasOwnProperty.call(
            dirty,
            s.key,
          );
          const setField = (field: string, value: unknown) => {
            const updated = { ...data, [field]: value };
            setSetting(leafKey, updated);
          };
          const enabled = data.enabled !== false;
          return (
            <div
              key={s.key}
              role="group"
              aria-labelledby={providerHeadingId}
              className={cn(
                "min-w-0 rounded-xl border p-4 space-y-4",
                enabled
                  ? "border-border bg-card"
                  : "border-border/50 bg-muted/30 opacity-60",
              )}
            >
              <div className="flex items-center justify-between">
                <div>
                  <h3
                    id={providerHeadingId}
                    className="font-display text-sm font-semibold text-foreground"
                  >
                    {displayName}
                  </h3>
                  <p className="font-mono text-xs text-muted-foreground">
                    {s.key}
                  </p>
                </div>
                <div className="flex items-center gap-3">
                  {providerIsDirty && (
                    <span className="font-mono text-[10px] font-medium uppercase tracking-wide text-warning">
                      Modified
                    </span>
                  )}
                  <Toggle
                    id={enabledId}
                    ariaLabel={`Enable ${displayName} provider`}
                    value={enabled}
                    onChange={(v) => setField("enabled", v)}
                  />
                </div>
              </div>
              <div className="grid min-w-0 grid-cols-1 gap-3 lg:grid-cols-2">
                <Field label="Base URL" htmlFor={baseUrlId}>
                  <Input
                    id={baseUrlId}
                    value={(data.base_url as string) ?? ""}
                    onChange={(e) => setField("base_url", e.target.value)}
                    placeholder="https://api.example.com/v1"
                    className="font-mono text-xs"
                  />
                </Field>
                <Field label="Protocol" htmlFor={protocolId}>
                  <SettingSelect
                    id={protocolId}
                    value={(data.protocol as string) ?? "auto"}
                    options={[
                      { value: "auto", label: "Auto" },
                      { value: "chat", label: "Chat Completions" },
                      { value: "responses", label: "Responses API" },
                    ]}
                    onChange={(v) => setField("protocol", v)}
                  />
                </Field>
                <Field
                  label="API Key"
                  hint="Masked for security"
                  htmlFor={apiKeyId}
                  hintId={apiKeyHintId}
                >
                  <MaskedInput
                    id={apiKeyId}
                    ariaDescribedBy={apiKeyHintId}
                    revealLabel={`${displayName} API key`}
                    value={(data.api_key as string) ?? ""}
                    onChange={(v) => setField("api_key", v)}
                    placeholder="sk-..."
                  />
                </Field>
                <Field
                  label="Default Model"
                  hint={modelHint}
                  htmlFor={defaultModelId}
                  hintId={modelHint ? defaultModelHintId : undefined}
                >
                  <SettingModelPicker
                    id={defaultModelId}
                    ariaDescribedBy={
                      modelHint ? defaultModelHintId : undefined
                    }
                    value={currentModelIsAvailable ? currentModel : ""}
                    options={modelOptions}
                    onChange={(v) => setField("default_model", v)}
                    triggerClassName="w-full"
                    ariaLabel={`${displayName} default model`}
                    searchAriaLabel={`Search ${displayName} models`}
                    disabled={modelOptions.length === 0}
                    placeholder={
                      modelOptions.length === 0
                        ? "No enabled models"
                        : "Select a model"
                    }
                    ariaInvalid={Boolean(
                      modelOptions.length > 0 &&
                        currentModel &&
                        !currentModelIsAvailable,
                    )}
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
