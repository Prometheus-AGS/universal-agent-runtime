import { useCallback, useEffect, useId, useRef, useState } from "react";
import {
  AlertCircle,
  AlertTriangle,
  Bot,
  CheckCircle2,
  Info,
  Loader2,
  Zap,
} from "lucide-react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";
import type {
  GovernanceEffectiveState,
  GovernanceRuntimeStatus,
  GovernanceStatusReason,
} from "../../api/settings-api";
import { useGovernanceStatus } from "../../model/use-governance-status";
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

const GOVERNANCE_REASON_LABELS: Record<GovernanceStatusReason, string> = {
  initialization_incomplete: "Runtime initialization is not complete.",
  configured_host_not_allowed:
    "The configured listener host is not localhost or 127.0.0.1.",
  authentication_unverified: "The installed authentication mode is unknown.",
  jwt_required: "JWT authentication is active for this process.",
  ingress_inventory_unsealed:
    "The runtime has not sealed its tool-capable ingress inventory.",
  ingress_proof_missing: "A tool-capable ingress is missing its bound-address proof.",
  bound_ingress_not_loopback:
    "At least one tool-capable ingress is bound beyond this device.",
  persistence_unavailable:
    "The saved governance preference could not be read or updated.",
};

const GOVERNANCE_SETTING_LABELS: Record<string, string> = {
  "governance.enabled": "Enforce tool governance",
  "governance.default_mode": "Default authorization mode",
  "governance.allowed_actions": "Globally allowed actions",
  "governance.policy_reload_enabled": "Hot policy reload",
};

function governanceSettingLabel(key: string) {
  return GOVERNANCE_SETTING_LABELS[key] ?? key;
}

function governanceStateLabel(state: GovernanceEffectiveState | undefined) {
  switch (state) {
    case "required":
      return "Required";
    case "on":
      return "On";
    case "off":
      return "Off";
    default:
      return "Unknown";
  }
}

export function GovernancePanel() {
  const {
    values,
    dirty,
    loading: settingsLoading,
    saving,
    error: settingsError,
    setSetting,
    saveAll,
    reload,
  } = useSettings("governance");
  const {
    status,
    loading: statusLoading,
    error: statusError,
    refresh: refreshStatus,
  } = useGovernanceStatus();
  const [feedback, setFeedback] = useState<{
    kind: "confirmed" | "changed" | "error";
    message: string;
  } | null>(null);
  const [announcement, setAnnouncement] = useState("");
  const lastAnnouncedEffectiveState =
    useRef<GovernanceEffectiveState | null>(null);
  const wasSaving = useRef(false);
  const masterId = useId();
  const masterDescriptionId = useId();
  const masterLockReasonId = useId();
  const modeId = useId();
  const modeHintId = useId();
  const reloadId = useId();
  const policyDescriptionId = useId();

  useEffect(() => {
    const effectiveState = statusError ? undefined : status?.effective_state;
    if (!effectiveState || effectiveState === "unknown") return;
    if (lastAnnouncedEffectiveState.current === null) {
      lastAnnouncedEffectiveState.current = effectiveState;
      return;
    }
    if (lastAnnouncedEffectiveState.current === effectiveState || saving) return;
    lastAnnouncedEffectiveState.current = effectiveState;
    setAnnouncement(
      `Tool governance is now ${governanceStateLabel(effectiveState)}.`,
    );
  }, [saving, status?.effective_state, statusError]);

  const set = useCallback(
    (key: string, value: unknown) => {
      setFeedback(null);
      setSetting(`governance.${key}`, value);
    },
    [setSetting],
  );
  const mode =
    (values["governance.default_mode"] as string | undefined) ?? "permit_all";
  const allowed =
    (values["governance.allowed_actions"] as string[] | undefined) ?? [];
  const hasMasterDraft = Object.prototype.hasOwnProperty.call(
    dirty,
    "governance.enabled",
  );
  const hasDirty = Object.keys(dirty).length > 0;
  const statusUnknown =
    !status || status.effective_state === "unknown" || Boolean(statusError);
  const settingsUnavailable = settingsLoading || Boolean(settingsError);
  const mutationUnavailable = status?.mutation_available === false;
  const governanceRequired = status?.effective_state === "required";
  const presentedEnabled = governanceRequired
    ? true
    : mutationUnavailable
      ? (status?.effective_enabled ?? true)
      : hasMasterDraft
        ? Boolean(values["governance.enabled"])
        : (status?.effective_enabled ?? true);
  const masterAriaDisabled =
    statusUnknown ||
    settingsUnavailable ||
    mutationUnavailable ||
    governanceRequired;
  const policyDisabled =
    saving ||
    statusUnknown ||
    settingsUnavailable ||
    mutationUnavailable ||
    (!governanceRequired && !presentedEnabled);

  const announceObservedStatus = useCallback(
    (
      observedStatus: GovernanceRuntimeStatus | undefined,
      fallback: string,
    ) => {
      const effectiveState = observedStatus?.effective_state;
      if (
        effectiveState &&
        effectiveState !== "unknown" &&
        lastAnnouncedEffectiveState.current !== effectiveState
      ) {
        lastAnnouncedEffectiveState.current = effectiveState;
        setAnnouncement(
          `Tool governance is now ${governanceStateLabel(effectiveState)}.`,
        );
        return;
      }
      setAnnouncement(fallback);
    },
    [],
  );

  useEffect(() => {
    if (saving && !wasSaving.current) {
      setAnnouncement(
        hasMasterDraft
          ? presentedEnabled
            ? "Turning tool governance on…"
            : "Turning tool governance off…"
          : "Saving governance settings…",
      );
    }
    wasSaving.current = saving;
  }, [hasMasterDraft, presentedEnabled, saving]);

  const toggleAction = (action: string) => {
    const next = allowed.includes(action)
      ? allowed.filter((candidate) => candidate !== action)
      : [...allowed, action];
    set("allowed_actions", next);
  };

  const handleSave = useCallback(async () => {
    setFeedback(null);
    try {
      const response = await saveAll();
      if (!response) return;
      const updated = response.results?.filter(
        (result) => result.status === "updated",
      );
      const appliedLabels = (updated ?? []).map((result) =>
        governanceSettingLabel(result.key),
      );
      const retainedLabels = (response.retained_draft_keys ?? []).map(
        governanceSettingLabel,
      );
      const observedStatus =
        response.observed_governance_status ?? response.governance_status;
      const outcome =
        response.governance_outcome ??
        (response.status === "updated" ? "confirmed" : "partial");
      if (outcome === "confirmed") {
        const message = "Governance settings saved and confirmed.";
        setFeedback({ kind: "confirmed", message });
        announceObservedStatus(observedStatus, message);
      } else if (outcome === "changed_elsewhere") {
        const message = `Settings saved, then changed elsewhere. Tool governance is now ${governanceStateLabel(
          response.observed_governance_status?.effective_state,
        )}.`;
        setFeedback({ kind: "changed", message });
        announceObservedStatus(observedStatus, message);
      } else if (outcome === "unknown") {
        setFeedback({
          kind: "error",
          message:
            "The previous runtime stopped before the save outcome could be verified. Refresh to try again.",
        });
        setAnnouncement("");
      } else {
        const message = `${
          appliedLabels.length > 0
            ? `Applied: ${appliedLabels.join(", ")}. `
            : "No governance settings were applied. "
        }${
          retainedLabels.length > 0
            ? `Still drafts: ${retainedLabels.join(", ")}.`
            : "No submitted drafts remain."
        }`;
        setFeedback({ kind: "error", message });
        announceObservedStatus(observedStatus, "");
      }
    } catch {
      setAnnouncement("");
    }
  }, [announceObservedStatus, saveAll]);

  const handleRefresh = useCallback(async () => {
    setFeedback(null);
    await Promise.allSettled([reload(), refreshStatus()]);
  }, [reload, refreshStatus]);

  return (
    <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
      <PanelHeader
        title="Governance"
        subtitle="Tool policies, approvals, and enforcement"
        saving={saving}
        loading={settingsLoading || statusLoading}
        saveDisabled={
          !hasDirty ||
          statusUnknown ||
          settingsUnavailable ||
          mutationUnavailable
        }
        onSave={() => void handleSave()}
        onReload={() => void handleRefresh()}
      />
      <div className="min-w-0 flex-1 space-y-5 overflow-y-auto px-4 py-5 sm:px-6">
        {(settingsLoading || statusLoading) && !status && (
          <div className="flex items-center gap-2">
            <Loader2 size={15} className="animate-spin text-muted-foreground" />
            <span className="font-mono text-xs text-muted-foreground">
              Verifying runtime governance…
            </span>
          </div>
        )}

        {statusError && (
          <div
            role="alert"
            aria-atomic="true"
            className="flex min-w-0 items-start gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3 text-destructive"
          >
            <AlertCircle size={15} className="mt-0.5 shrink-0" />
            <p className="min-w-0 font-mono text-xs leading-relaxed break-words">
              Runtime governance status could not be verified. Refresh to try
              again. {statusError}
            </p>
          </div>
        )}

        {settingsError && (
          <div
            role="alert"
            aria-atomic="true"
            className="flex min-w-0 items-start gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3 text-destructive"
          >
            <AlertCircle size={15} className="mt-0.5 shrink-0" />
            <p className="min-w-0 font-mono text-xs leading-relaxed break-words">
              Governance settings could not be loaded or saved. Refresh to try
              again. {settingsError}
            </p>
          </div>
        )}

        <section
          aria-labelledby={`${masterId}-heading`}
          className="min-w-0 rounded-xl border border-border bg-card p-4"
        >
          <div className="grid min-w-0 grid-cols-1 items-center gap-4 min-[360px]:grid-cols-[minmax(0,1fr)_auto]">
            <div className="min-w-0 space-y-1.5">
              <div className="flex min-w-0 flex-wrap items-center gap-2">
                <Label
                  id={`${masterId}-heading`}
                  htmlFor={statusUnknown ? undefined : masterId}
                  className="font-display text-sm font-semibold text-foreground"
                >
                  Enforce tool governance
                </Label>
                <span
                  className={cn(
                    "rounded-full border px-2 py-0.5 font-mono text-xs font-semibold",
                    !statusUnknown &&
                      status?.effective_state === "on" &&
                      "border-success/40 bg-success/10 text-success",
                    !statusUnknown &&
                      status?.effective_state === "off" &&
                      "border-warning bg-warning/10 text-foreground",
                    !statusUnknown &&
                      status?.effective_state === "required" &&
                      "border-primary/40 bg-primary/10 text-primary",
                    (statusUnknown || mutationUnavailable) &&
                      "border-border bg-muted text-muted-foreground",
                  )}
                >
                  {governanceStateLabel(
                    statusUnknown ? undefined : status?.effective_state,
                  )}
                </span>
              </div>
              <p
                id={masterDescriptionId}
                className="max-w-[70ch] font-body text-sm leading-relaxed text-muted-foreground"
              >
                Applies Cedar authorization, run-policy restrictions, and risk
                approval before each tool call.
              </p>
            </div>
            {statusUnknown ? (
              <span className="font-mono text-xs text-muted-foreground">
                Unavailable
              </span>
            ) : (
              <Toggle
                id={masterId}
                value={presentedEnabled}
                onChange={(value) => set("enabled", value)}
                disabled={saving}
                ariaDisabled={masterAriaDisabled}
                ariaLabel="Enforce tool governance"
                ariaDescribedBy={
                  governanceRequired || mutationUnavailable
                    ? `${masterDescriptionId} ${masterLockReasonId}`
                    : masterDescriptionId
                }
              />
            )}
          </div>

          {hasMasterDraft &&
            status &&
            !governanceRequired &&
            !mutationUnavailable && (
              <div className="mt-3 flex min-w-0 items-start gap-2 rounded-lg bg-muted/40 px-3 py-2 text-muted-foreground">
                <Info size={14} className="mt-0.5 shrink-0" />
                <p className="min-w-0 font-mono text-xs leading-relaxed">
                  {presentedEnabled
                    ? "After Save, policy checks and approval prompts resume."
                    : "After Save, all available tools can run without Cedar policies, run-policy restrictions, or approval prompts."}{" "}
                  Effective governance remains{" "}
                  {governanceStateLabel(status.effective_state)} until
                  confirmed.
                </p>
              </div>
            )}
        </section>

        {!statusUnknown && status?.effective_state === "off" && (
          <div
            role="note"
            className="flex min-w-0 items-start gap-2 rounded-lg border border-foreground/50 bg-warning/10 px-4 py-3 text-foreground"
          >
            <AlertTriangle
              size={15}
              className="mt-0.5 shrink-0 text-warning"
            />
            <p className="min-w-0 font-body text-sm leading-relaxed">
              All available tools can run without Cedar policies, run-policy
              restrictions, or approval prompts.
            </p>
          </div>
        )}

        {!statusUnknown &&
          (governanceRequired || mutationUnavailable) &&
          status && (
          <div
            id={masterLockReasonId}
            className="min-w-0 rounded-lg border border-border bg-muted/30 px-4 py-3"
          >
            <p className="font-body text-sm font-medium text-foreground">
              {governanceRequired && mutationUnavailable
                ? "Governance is required and settings are unavailable."
                : governanceRequired
                  ? "Governance is required for this runtime."
                  : "Governance is enforced, but settings are unavailable."}
            </p>
            <ul className="mt-2 space-y-1 pl-5 font-body text-sm text-muted-foreground">
              {status.reasons.map((reason) => (
                <li key={reason} className="list-disc break-words">
                  {GOVERNANCE_REASON_LABELS[reason]}
                </li>
              ))}
            </ul>
            {governanceRequired && (
              <p className="mt-2 font-body text-sm leading-relaxed text-muted-foreground">
                Change the active listener or JWT configuration named above,
                restore settings persistence when listed, then restart UAR
                before turning governance Off.
              </p>
            )}
          </div>
          )}

        <fieldset
          disabled={policyDisabled}
          aria-describedby={policyDescriptionId}
          className={cn(
            "min-w-0 space-y-5 rounded-xl border border-border p-4",
            policyDisabled && "opacity-60",
          )}
        >
          <legend className="px-1 font-display text-sm font-semibold text-foreground">
            Policy behavior when governance is on
          </legend>
          <p
            id={policyDescriptionId}
            className="max-w-[70ch] font-body text-sm leading-relaxed text-muted-foreground"
          >
            These controls remain saved while governance is Off and apply again
            when enforcement is turned On.
          </p>
          <Field
            label="Default Authorization Mode"
            hint="Applied when no specific Cedar policy matches"
            htmlFor={modeId}
            hintId={modeHintId}
          >
            <SettingSelect
              id={modeId}
              ariaDescribedBy={modeHintId}
              value={mode}
              options={[
                { value: "permit_all", label: "Permit All (default allow)" },
                { value: "deny_all", label: "Deny All (default deny)" },
                { value: "custom", label: "Custom (policy files only)" },
              ]}
              onChange={(value) => set("default_mode", value)}
            />
          </Field>
          {mode !== "permit_all" && (
            <div className="min-w-0 space-y-2">
              <p className="font-mono text-xs font-medium tracking-wide text-muted-foreground uppercase">
                Globally Allowed Actions
              </p>
              <div className="space-y-2">
                {GOVERNANCE_ACTIONS.map(({ value, label }) => (
                  <label
                    key={value}
                    className="flex min-w-0 cursor-pointer flex-wrap items-center gap-x-3 gap-y-1 rounded-lg border border-border bg-card px-4 py-2.5"
                  >
                    <input
                      type="checkbox"
                      checked={allowed.includes(value)}
                      onChange={() => toggleAction(value)}
                      className="shrink-0 accent-primary"
                    />
                    <span className="min-w-0 flex-1 font-body text-sm text-foreground">
                      {label}
                    </span>
                    <span className="min-w-0 font-mono text-xs break-all text-muted-foreground">
                      {value}
                    </span>
                  </label>
                ))}
              </div>
            </div>
          )}
          <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-4 rounded-lg border border-border bg-card px-4 py-3">
            <div className="min-w-0">
              <Label
                htmlFor={reloadId}
                className="font-body text-sm font-medium text-foreground"
              >
                Hot policy reload
              </Label>
              <p className="font-body text-sm leading-relaxed text-muted-foreground">
                Reload Cedar policy files without restarting the server.
              </p>
            </div>
            <Toggle
              id={reloadId}
              ariaLabel="Hot policy reload"
              value={
                (values["governance.policy_reload_enabled"] as
                  | boolean
                  | undefined) ?? true
              }
              onChange={(value) => set("policy_reload_enabled", value)}
            />
          </div>
        </fieldset>

        {feedback && (
          <div
            className={cn(
              "flex min-w-0 items-start gap-2 rounded-lg border px-4 py-3",
              feedback.kind === "confirmed"
                ? "border-success/40 bg-success/10 text-success"
                : feedback.kind === "changed"
                  ? "border-border bg-muted/40 text-foreground"
                  : "border-destructive/40 bg-destructive/10 text-destructive",
            )}
            role={feedback.kind === "error" ? "alert" : undefined}
            aria-atomic={feedback.kind === "error" ? "true" : undefined}
          >
            {feedback.kind === "confirmed" ? (
              <CheckCircle2 size={15} className="mt-0.5 shrink-0" />
            ) : feedback.kind === "changed" ? (
              <Info size={15} className="mt-0.5 shrink-0" />
            ) : (
              <AlertCircle size={15} className="mt-0.5 shrink-0" />
            )}
            <p className="min-w-0 font-mono text-xs leading-relaxed">
              {feedback.message}
            </p>
          </div>
        )}
        <p
          role="status"
          aria-live="polite"
          aria-atomic="true"
          className="sr-only"
        >
          {announcement ||
            ((settingsLoading || statusLoading) && !status
              ? "Verifying runtime governance…"
              : "")}
        </p>
      </div>
    </div>
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
