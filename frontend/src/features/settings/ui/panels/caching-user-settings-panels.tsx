import { useCallback, useEffect, useId, useRef, useState } from "react";
import {
  AlertCircle,
  Check,
  Loader2,
  RefreshCw,
  Save,
  User,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { friendlyError } from "@/lib/utils";
import type { UarUserSettings } from "../../api/user-settings-api";
import { useUserJwtSettings } from "../../model/use-user-jwt-settings";
import { useSettings } from "../../model/use-settings";
import { SettingsHint } from "../generic-schema-panel";
import {
  ErrorBanner,
  PanelHeader,
  SavedBanner,
  SettingSelect,
  Toggle,
} from "../settings-primitives";

export function PromptCachingPanel() {
  const enabledId = useId();
  const descriptionId = useId();
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
  } = useSettings("prompt_caching");
  const [saved, setSaved] = useState(false);
  const savedTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const enabledKey = "prompt_caching.enabled";
  const available = Object.prototype.hasOwnProperty.call(settings, enabledKey);
  const dirtyCount = Object.keys(dirty).length;
  const enabled = available ? Boolean(values[enabledKey]) : false;

  useEffect(
    () => () => {
      if (savedTimer.current) clearTimeout(savedTimer.current);
    },
    [],
  );

  const save = useCallback(async () => {
    try {
      await saveAll();
      setSaved(true);
      if (savedTimer.current) clearTimeout(savedTimer.current);
      savedTimer.current = setTimeout(() => setSaved(false), 2500);
    } catch {
      // The settings store retains the actionable error and dirty draft.
    }
  }, [saveAll]);

  const statusText = saving
    ? "Saving global default…"
    : refreshing
      ? "Refreshing server value…"
      : dirtyCount > 0
        ? "Unsaved changes"
        : available
          ? `Global default is ${enabled ? "On" : "Off"}`
          : undefined;

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <PanelHeader
        title="Prompt Caching"
        subtitle="Anthropic explicit caching and OpenAI provider-managed caching"
        saving={saving}
        loading={refreshing}
        saveDisabled={
          !available || loading || refreshing || saving || dirtyCount === 0
        }
        onSave={() => void save()}
        onReload={() => void reload()}
        statusText={statusText}
      />
      <div className="flex-1 overflow-y-auto px-6 py-5">
        {loading && !available && (
          <div
            className="flex items-center gap-2"
            role="status"
            aria-live="polite"
          >
            <Loader2 size={15} className="animate-spin text-muted-foreground" />
            <span className="font-mono text-xs text-muted-foreground">
              Loading prompt-caching settings…
            </span>
          </div>
        )}

        {!available && !loading && (
          <div
            role="alert"
            aria-atomic="true"
            className="flex items-start justify-between gap-4 rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3"
          >
            <div className="flex min-w-0 items-start gap-2">
              <AlertCircle
                size={14}
                className="mt-0.5 shrink-0 text-destructive"
              />
              <div>
                <p className="font-mono text-xs font-medium text-destructive">
                  Prompt-caching settings are unavailable.
                </p>
                <p className="mt-1 font-body text-xs text-destructive">
                  {error
                    ? friendlyError(error)
                    : "The server did not return a registered global setting."}{" "}
                  Check the runtime connection, then try again.
                </p>
              </div>
            </div>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => void reload()}
              disabled={refreshing}
              className="shrink-0 gap-1.5"
            >
              <RefreshCw
                size={13}
                className={refreshing ? "animate-spin" : undefined}
              />
              Retry
            </Button>
          </div>
        )}

        {available && (
          <div className="space-y-6">
            <ErrorBanner error={error} />
            <SavedBanner show={saved} />
            <SettingsHint id="prompt_caching">
              Prompt caching reuses stable prompt prefixes to reduce latency and
              token costs. Anthropic receives explicit cache controls when the
              effective setting is On. OpenAI manages eligible prompt caches
              automatically.
            </SettingsHint>

            <p className="font-mono text-xs text-muted-foreground mb-3">
              This switch is the system-wide fallback. A user or session can
              override it; a per-request override has the highest precedence.
            </p>

            <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3 mb-1">
              <div>
                <label
                  htmlFor={enabledId}
                  className="font-mono text-xs font-medium text-foreground"
                >
                  Enable Prompt Caching (Global Default)
                </label>
                <p
                  id={descriptionId}
                  className="font-body text-xs text-muted-foreground"
                >
                  Off by default. This control affects explicit Anthropic cache
                  annotations; OpenAI remains provider-managed.
                </p>
              </div>
              <Toggle
                id={enabledId}
                ariaLabel="Enable prompt caching as the global default"
                ariaDescribedBy={descriptionId}
                value={enabled}
                onChange={(value) => setSetting(enabledKey, value)}
                disabled={refreshing || saving}
              />
            </div>

            <div className="mt-4 rounded-lg border border-border bg-muted/30 px-4 py-3">
              <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground mb-1">
                Supported Providers
              </p>
              <div className="flex gap-2 mt-1">
                <span className="inline-flex items-center rounded-md border border-border bg-card px-2 py-0.5 font-mono text-xs font-medium text-foreground">
                  Anthropic
                </span>
                <span className="inline-flex items-center rounded-md border border-border bg-card px-2 py-0.5 font-mono text-xs font-medium text-foreground">
                  OpenAI (auto)
                </span>
              </div>
              <p className="font-mono text-xs text-muted-foreground mt-2">
                Effective priority: request → session → user → global.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// User Settings Panel (JWT-gated)
// =============================================================================

export function UserSettingsPanel() {
  const apiKey =
    (import.meta as unknown as { env: Record<string, string> }).env
      .VITE_UAR_API_KEY ?? "";
  const isJwt = apiKey.startsWith("ey");

  const {
    settings,
    loading,
    saving,
    error,
    load,
    save: saveRemote,
  } = useUserJwtSettings();
  const [draft, setDraft] = useState<UarUserSettings | null>(null);
  const [saved, setSaved] = useState(false);
  const savedTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (!settings) return;
    queueMicrotask(() => {
      setDraft(settings as UarUserSettings);
    });
  }, [settings]);

  const save = async () => {
    if (!draft || !isJwt) return;
    try {
      await saveRemote({
        prompt_caching_enabled: draft.prompt_caching_enabled,
      });
      setSaved(true);
      if (savedTimer.current) clearTimeout(savedTimer.current);
      savedTimer.current = setTimeout(() => setSaved(false), 2000);
    } catch {
      /* error from store */
    }
  };
  const dirty =
    draft !== null &&
    settings !== null &&
    draft.prompt_caching_enabled !== settings.prompt_caching_enabled;

  if (!isJwt) {
    return (
      <div className="flex flex-1 flex-col overflow-y-auto">
        <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
          <div>
            <h2 className="font-mono text-sm font-semibold text-foreground">
              User Settings
            </h2>
            <p className="font-mono text-xs text-muted-foreground">
              JWT authentication required
            </p>
          </div>
        </div>
        <div className="flex flex-1 items-center justify-center p-8">
          <div className="flex flex-col items-center gap-3 text-center">
            <User size={32} className="text-muted-foreground/40" />
            <p className="font-mono text-xs text-muted-foreground">
              Per-user settings are only available when{" "}
              <code className="rounded bg-muted px-1">VITE_UAR_API_KEY</code> is
              a JWT Bearer token.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col overflow-y-auto">
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-mono text-sm font-semibold text-foreground">
            User Settings
          </h2>
          <p className="font-mono text-xs text-muted-foreground">
            Per-user prompt-caching preferences for{" "}
            <span className="font-medium text-foreground">
              {draft?.user_id ?? settings?.user_id ?? "…"}
            </span>
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            className="font-mono text-xs"
            onClick={() => {
              void load();
            }}
            disabled={loading}
          >
            {loading ? (
              <Loader2 size={12} className="animate-spin mr-1" />
            ) : (
              <RefreshCw size={12} className="mr-1" />
            )}
            Reload
          </Button>
          <Button
            size="sm"
            className="font-mono text-xs"
            onClick={() => {
              void save();
            }}
            disabled={saving || loading || !draft || !dirty}
          >
            {saved ? (
              <>
                <Check size={12} className="mr-1" /> Saved
              </>
            ) : saving ? (
              <>
                <Loader2 size={12} className="animate-spin mr-1" /> Saving…
              </>
            ) : (
              <>
                <Save size={12} className="mr-1" /> Save
              </>
            )}
          </Button>
        </div>
      </div>

      {error && (
        <div className="mx-6 mt-4 flex items-center gap-2 rounded-md border border-destructive/40 bg-destructive/10 px-4 py-2">
          <AlertCircle size={13} className="text-destructive" />
          <span className="font-mono text-xs text-destructive">
            {friendlyError(error)}
          </span>
        </div>
      )}

      <div className="space-y-4 p-6">
        <div className="rounded-lg border border-border bg-card px-4 py-3">
          <label
            htmlFor="user-prompt-caching"
            className="font-mono text-xs font-medium text-foreground"
          >
            Prompt Caching
          </label>
          <p
            id="user-prompt-caching-description"
            className="mb-3 font-body text-xs text-muted-foreground"
          >
            Choose Inherit to use the system-wide global default for your
            account.
          </p>
          <div className="flex items-center gap-3">
            <SettingSelect
              id="user-prompt-caching"
              ariaDescribedBy="user-prompt-caching-description"
              value={
                draft?.prompt_caching_enabled === null
                  ? "inherit"
                  : draft?.prompt_caching_enabled
                    ? "on"
                    : "off"
              }
              options={[
                { value: "inherit", label: "Inherit" },
                { value: "on", label: "On" },
                { value: "off", label: "Off" },
              ]}
              onChange={(value) =>
                setDraft((current) =>
                  current
                    ? {
                        ...current,
                        prompt_caching_enabled:
                          value === "inherit" ? null : value === "on",
                      }
                    : current,
                )
              }
              disabled={!draft || loading || saving}
              triggerClassName="w-40"
            />
            <span
              role="status"
              aria-live="polite"
              className="font-mono text-xs text-muted-foreground"
            >
              {draft?.prompt_caching_enabled === null
                ? "Using global default"
                : `User override is ${draft?.prompt_caching_enabled ? "On" : "Off"}`}
            </span>
          </div>
        </div>

        {(draft?.updated_at ?? settings?.updated_at) && (
          <p className="font-mono text-xs text-muted-foreground">
            Last updated:{" "}
            {new Date(
              (draft?.updated_at ?? settings?.updated_at) as string,
            ).toLocaleString()}
          </p>
        )}
      </div>
    </div>
  );
}
