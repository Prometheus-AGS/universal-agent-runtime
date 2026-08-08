import { useCallback, useState } from "react";
import { Bot, Loader2, Zap } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";
import { useSettings } from "../../model/use-settings";
import { NamespacePanel } from "../generic-schema-panel";
import {
  ErrorBanner,
  Field,
  PanelHeader,
  SavedBanner,
  SettingSelect,
  Toggle,
} from "../settings-primitives";
import {
  buildGlobalResiliencePreview,
  mergeAgentResiliencePreview,
} from "./resilience-preview";

const IC_BACKENDS = [
  { value: "rules", label: "Rules" },
  { value: "tfidf", label: "TF-IDF" },
  { value: "wasm", label: "WASM Component" },
  { value: "hybrid", label: "Hybrid" },
  { value: "localembedding", label: "Local Embeddings" },
  { value: "llm", label: "LLM" },
];

export function IntentClassifierPanel() {
  return (
    <NamespacePanel namespace="intent_classifier" title="Intent Classifier">
      {({ val, set }) => (
        <>
          <Field label="Backend">
            <SettingSelect
              value={(val("backend") as string) ?? "rules"}
              options={IC_BACKENDS}
              onChange={(v) => set("backend", v)}
            />
          </Field>
          <div className="grid grid-cols-2 gap-4">
            <Field label="Top-K Results">
              <Input
                type="number"
                min={1}
                value={(val("topk") as number) ?? 3}
                onChange={(e) => set("topk", parseInt(e.target.value))}
                className="font-mono text-xs"
              />
            </Field>
            <Field label="Accept Threshold" hint="0.0 – 1.0">
              <Input
                type="number"
                step={0.05}
                min={0.0}
                max={1.0}
                value={(val("accept_threshold") as number) ?? 0.7}
                onChange={(e) =>
                  set("accept_threshold", parseFloat(e.target.value))
                }
                className="font-mono text-xs"
              />
            </Field>
            <Field
              label="Margin Threshold"
              hint="0.0 – 1.0 (confidence margin between top results)"
            >
              <Input
                type="number"
                step={0.05}
                min={0.0}
                max={1.0}
                value={(val("margin_threshold") as number) ?? 0.1}
                onChange={(e) =>
                  set("margin_threshold", parseFloat(e.target.value))
                }
                className="font-mono text-xs"
              />
            </Field>
          </div>
          {(val("backend") as string) === "wasm" && (
            <Field label="WASM Component Path">
              <Input
                value={(val("wasm_component_path") as string) ?? ""}
                onChange={(e) =>
                  set("wasm_component_path", e.target.value || null)
                }
                placeholder="/path/to/component.wasm"
                className="font-mono text-xs"
              />
            </Field>
          )}
        </>
      )}
    </NamespacePanel>
  );
}

// --- Governance -------------------------------------------------------------

const GOVERNANCE_ACTIONS = [
  { value: "execute_tool", label: "Execute Tool" },
  { value: "call_llm", label: "Call LLM" },
  { value: "spawn_agent", label: "Spawn Agent" },
  { value: "collaborate", label: "Collaborate (agent-to-agent)" },
  { value: "access_knowledge", label: "Access Knowledge Base" },
];

export function GovernancePanel() {
  return (
    <NamespacePanel
      namespace="governance"
      title="Governance"
      subtitle="Cedar-policy based authorization defaults"
    >
      {({ val, set }) => {
        const mode = (val("default_mode") as string) ?? "permit_all";
        const allowed = (val("allowed_actions") as string[]) ?? [];
        const toggleAction = (action: string) => {
          const next = allowed.includes(action)
            ? allowed.filter((a) => a !== action)
            : [...allowed, action];
          set("allowed_actions", next);
        };
        return (
          <>
            <Field
              label="Default Authorization Mode"
              hint="Applied when no specific Cedar policy matches"
            >
              <SettingSelect
                value={mode}
                options={[
                  { value: "permit_all", label: "Permit All (default allow)" },
                  { value: "deny_all", label: "Deny All (default deny)" },
                  { value: "custom", label: "Custom (policy files only)" },
                ]}
                onChange={(v) => set("default_mode", v)}
              />
            </Field>
            {mode !== "permit_all" && (
              <div className="space-y-2">
                <Label className="font-mono text-xs font-medium text-muted-foreground uppercase tracking-wide">
                  Globally Allowed Actions
                </Label>
                <div className="space-y-2">
                  {GOVERNANCE_ACTIONS.map(({ value, label }) => (
                    <label
                      key={value}
                      className="flex cursor-pointer items-center gap-3 rounded-lg border border-border bg-card px-4 py-2.5"
                    >
                      <input
                        type="checkbox"
                        checked={allowed.includes(value)}
                        onChange={() => toggleAction(value)}
                        className="accent-primary"
                      />
                      <span className="font-mono text-xs text-foreground">
                        {label}
                      </span>
                      <span className="ml-auto font-mono text-xs text-muted-foreground">
                        {value}
                      </span>
                    </label>
                  ))}
                </div>
              </div>
            )}
            <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
              <div>
                <p className="font-mono text-xs font-medium text-foreground">
                  Hot Policy Reload
                </p>
                <p className="font-mono text-xs text-muted-foreground">
                  Reload Cedar policy files without restarting the server
                </p>
              </div>
              <Toggle
                value={(val("policy_reload_enabled") as boolean) ?? true}
                onChange={(v) => set("policy_reload_enabled", v)}
              />
            </div>
          </>
        );
      }}
    </NamespacePanel>
  );
}

// --- Agent Config -----------------------------------------------------------

const AGENT_CONTEXT_OPTS = [
  { value: "inherit", label: "Inherit Global" },
  { value: "sliding_window", label: "Sliding Window" },
  { value: "keep_first_last", label: "Keep First & Last" },
  { value: "progressive_summarization", label: "Progressive Summarization" },
  { value: "none", label: "None" },
];

const AGENT_GOV_OPTS = [
  { value: "inherit", label: "Inherit Global" },
  { value: "permit_all", label: "Permit All" },
  { value: "deny_all", label: "Deny All" },
];

const AGENT_RESILIENCE_MODE_OPTS = [
  { value: "inherit", label: "Inherit Global" },
  { value: "override", label: "Override" },
];

export function AgentConfigPanel() {
  const {
    values,
    settings,
    loading,
    saving,
    error,
    setSetting,
    saveAll,
    reload,
  } = useSettings("agent_config");
  const { values: resilienceValues } = useSettings("resilience");
  const [savedFlash, setSavedFlash] = useState(false);

  const agentEntries = Object.values(settings).sort((a, b) =>
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
        title="Agent Configuration"
        subtitle={`${agentEntries.length} agent(s) configured`}
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
        {agentEntries.map((s) => {
          const data = (values[s.key] ?? {}) as Record<string, unknown>;
          const agentId = s.key.replace("agent_config.", "");
          const setField = (field: string, value: unknown) =>
            setSetting(s.key, { ...data, [field]: value });
          const resilienceOverride = (data.resilience as Record<
            string,
            unknown
          >) ?? { mode: "inherit" };
          const setResilienceField = (field: string, value: unknown) =>
            setField("resilience", { ...resilienceOverride, [field]: value });
          const enabled = data.enabled !== false;
          const resilienceMode =
            (resilienceOverride.mode as string) ?? "inherit";
          const globalPreview = buildGlobalResiliencePreview(resilienceValues);
          const effectivePreview = mergeAgentResiliencePreview(
            globalPreview,
            resilienceOverride,
          );
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
                  <p className="font-display text-sm font-semibold text-foreground flex items-center gap-2">
                    <Bot size={14} className="text-muted-foreground" />
                    {s.name}
                  </p>
                  <p className="font-mono text-xs text-muted-foreground">
                    {agentId}
                  </p>
                </div>
                <Toggle
                  value={enabled}
                  onChange={(v) => setField("enabled", v)}
                />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <Field label="Context Strategy">
                  <SettingSelect
                    value={(data.context_strategy as string) ?? "inherit"}
                    options={AGENT_CONTEXT_OPTS}
                    onChange={(v) => setField("context_strategy", v)}
                  />
                </Field>
                <Field label="Governance Mode">
                  <SettingSelect
                    value={(data.governance_mode as string) ?? "inherit"}
                    options={AGENT_GOV_OPTS}
                    onChange={(v) => setField("governance_mode", v)}
                  />
                </Field>
              </div>
              <Field
                label="Allowed Tools"
                hint="Comma-separated tool names. Leave blank to allow all."
              >
                <Input
                  value={
                    Array.isArray(data.allowed_tools)
                      ? (data.allowed_tools as string[]).join(", ")
                      : ""
                  }
                  onChange={(e) =>
                    setField(
                      "allowed_tools",
                      e.target.value
                        ? e.target.value
                            .split(",")
                            .map((t) => t.trim())
                            .filter(Boolean)
                        : [],
                    )
                  }
                  placeholder="web_search, code_exec, … (blank = all)"
                  className="font-mono text-xs"
                />
              </Field>
              <div className="space-y-3 rounded-lg border border-border/60 bg-muted/20 p-3">
                <Field label="Resilience Policy">
                  <SettingSelect
                    value={resilienceMode}
                    options={AGENT_RESILIENCE_MODE_OPTS}
                    onChange={(v) => setResilienceField("mode", v)}
                  />
                </Field>
                {resilienceMode === "override" && (
                  <div className="grid grid-cols-2 gap-3">
                    <Field label="Request Timeout (ms)">
                      <Input
                        type="number"
                        min={1000}
                        value={
                          (resilienceOverride.request_timeout_ms as number) ??
                          ""
                        }
                        onChange={(e) =>
                          setResilienceField(
                            "request_timeout_ms",
                            e.target.value ? Number(e.target.value) : null,
                          )
                        }
                        placeholder={`${globalPreview.request_timeout_ms ?? ""}`}
                        className="font-mono text-xs"
                      />
                    </Field>
                    <Field label="Retry Max Attempts">
                      <Input
                        type="number"
                        min={0}
                        max={10}
                        value={
                          (resilienceOverride.retry_max_attempts as number) ??
                          ""
                        }
                        onChange={(e) =>
                          setResilienceField(
                            "retry_max_attempts",
                            e.target.value ? Number(e.target.value) : null,
                          )
                        }
                        placeholder={`${globalPreview.retry_max_attempts ?? ""}`}
                        className="font-mono text-xs"
                      />
                    </Field>
                    <Field label="Retry Base Delay (ms)">
                      <Input
                        type="number"
                        min={100}
                        value={
                          (resilienceOverride.retry_base_delay_ms as number) ??
                          ""
                        }
                        onChange={(e) =>
                          setResilienceField(
                            "retry_base_delay_ms",
                            e.target.value ? Number(e.target.value) : null,
                          )
                        }
                        placeholder={`${globalPreview.retry_base_delay_ms ?? ""}`}
                        className="font-mono text-xs"
                      />
                    </Field>
                    <Field label="Retry Max Delay (ms)">
                      <Input
                        type="number"
                        min={100}
                        value={
                          (resilienceOverride.retry_max_delay_ms as number) ??
                          ""
                        }
                        onChange={(e) =>
                          setResilienceField(
                            "retry_max_delay_ms",
                            e.target.value ? Number(e.target.value) : null,
                          )
                        }
                        placeholder={`${globalPreview.retry_max_delay_ms ?? ""}`}
                        className="font-mono text-xs"
                      />
                    </Field>
                  </div>
                )}
                <div className="space-y-1">
                  <p className="font-mono text-xs uppercase tracking-wider text-muted-foreground">
                    Effective policy preview
                  </p>
                  <pre className="max-h-36 overflow-auto rounded-md border border-border/60 bg-background/80 p-2 font-mono text-xs text-muted-foreground">
                    {JSON.stringify(effectivePreview, null, 2)}
                  </pre>
                </div>
              </div>
            </div>
          );
        })}
        {!loading && agentEntries.length === 0 && (
          <p className="font-mono text-xs text-muted-foreground">
            No agents configured yet.
          </p>
        )}
      </div>
    </div>
  );
}

// --- Skill Config -----------------------------------------------------------

export function SkillConfigPanel() {
  const {
    values,
    settings,
    loading,
    saving,
    error,
    setSetting,
    saveAll,
    reload,
  } = useSettings("skill_config");
  const [savedFlash, setSavedFlash] = useState(false);

  const skillEntries = Object.values(settings).sort((a, b) =>
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
        title="Skill Configuration"
        subtitle={`${skillEntries.length} built-in skill(s)`}
        saving={saving}
        loading={loading}
        onSave={() => void handleSave()}
        onReload={() => void reload()}
      />
      <div className="flex-1 overflow-y-auto px-6 py-5 space-y-2">
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
        {skillEntries.map((s) => {
          const data = (values[s.key] ?? {}) as Record<string, unknown>;
          const skillId = s.key.replace("skill_config.", "");
          const setField = (field: string, value: unknown) =>
            setSetting(s.key, { ...data, [field]: value });
          const enabled = data.enabled !== false;
          return (
            <div
              key={s.key}
              className={cn(
                "flex items-start gap-4 rounded-lg border px-4 py-3 transition-colors",
                enabled
                  ? "border-border bg-card"
                  : "border-border/40 bg-muted/20",
              )}
            >
              <div className="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-md bg-muted">
                <Zap
                  size={14}
                  className={cn(
                    enabled ? "text-primary" : "text-muted-foreground",
                  )}
                />
              </div>
              <div className="min-w-0 flex-1 space-y-1">
                <p
                  className={cn(
                    "font-mono text-sm font-medium",
                    enabled ? "text-foreground" : "text-muted-foreground",
                  )}
                >
                  {skillId}
                </p>
                <p className="font-mono text-xs text-muted-foreground">
                  {(data.description as string) ?? ""}
                </p>
                <Input
                  value={(data.description as string) ?? ""}
                  onChange={(e) => setField("description", e.target.value)}
                  placeholder="Override description…"
                  className="mt-1.5 h-7 font-mono text-xs"
                />
              </div>
              <Toggle
                value={enabled}
                onChange={(v) => setField("enabled", v)}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
