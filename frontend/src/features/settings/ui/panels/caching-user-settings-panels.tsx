import { useEffect, useRef, useState } from "react";
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
import { useUserJwtSettings } from "../../model/use-user-jwt-settings";
import { NamespacePanel } from "../generic-schema-panel";
import { Field, SettingSelect, Toggle } from "../settings-primitives";

export function PromptCachingPanel() {
  return (
    <NamespacePanel
      namespace="prompt_caching"
      title="Prompt Caching"
      hint="Prompt caching stores frequently-used prompt prefixes to reduce costs and latency. This is especially effective for system prompts and tool definitions that don't change between requests. Enable it to save on API costs with supported providers."
    >
      {({ val, set }) => (
        <>
          <p className="font-mono text-xs text-muted-foreground mb-3">
            Prompt caching reduces latency and token costs by reusing stable
            parts of the system prompt. Anthropic injects{" "}
            <code>cache_control</code> blocks; OpenAI caches automatically on
            eligible models.
          </p>

          <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3 mb-1">
            <div>
              <p className="font-mono text-xs font-medium text-foreground">
                Enable Prompt Caching (Global Default)
              </p>
              <p className="font-mono text-xs text-muted-foreground">
                System-wide default; users can override per-session via the chat
                toolbar.
              </p>
            </div>
            <Toggle
              value={(val("enabled") as boolean) ?? false}
              onChange={(v) => set("enabled", v)}
            />
          </div>

          <Field
            label="Cache Control Type"
            hint="Only 'ephemeral' is supported by Anthropic."
          >
            <SettingSelect
              value={(val("cache_control_type") as string) ?? "ephemeral"}
              options={[{ value: "ephemeral", label: "ephemeral" }]}
              onChange={(v) => set("cache_control_type", v)}
            />
          </Field>

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
              Priority: session override → user preference → agent setting →
              this global default.
            </p>
          </div>
        </>
      )}
    </NamespacePanel>
  );
}

// =============================================================================
// User Settings Panel (JWT-gated)
// =============================================================================

interface UarUserSettings {
  user_id: string;
  prompt_caching_enabled: boolean | null;
  preferred_scope: "session" | "user" | "agent";
  updated_at: string;
}

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
        preferred_scope: draft.preferred_scope,
      });
      setSaved(true);
      if (savedTimer.current) clearTimeout(savedTimer.current);
      savedTimer.current = setTimeout(() => setSaved(false), 2000);
    } catch {
      /* error from store */
    }
  };

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
            disabled={saving || loading || !draft}
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
        <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
          <div>
            <p className="font-mono text-xs font-medium text-foreground">
              Prompt Caching Enabled
            </p>
            <p className="font-mono text-xs text-muted-foreground">
              Override the global default for your account. Null = inherit
              global setting.
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="link"
              className="h-auto p-0 font-mono text-xs text-muted-foreground"
              onClick={() =>
                setDraft((s) =>
                  s ? { ...s, prompt_caching_enabled: null } : s,
                )
              }
            >
              reset
            </Button>
            <Toggle
              value={draft?.prompt_caching_enabled ?? false}
              onChange={(v) =>
                setDraft((s) => (s ? { ...s, prompt_caching_enabled: v } : s))
              }
              disabled={!draft}
            />
          </div>
        </div>

        <Field
          label="Preferred Scope"
          hint="Which level of settings takes precedence when no session override is present."
        >
          <SettingSelect
            value={draft?.preferred_scope ?? "session"}
            options={[
              { value: "session", label: "Session (per-conversation)" },
              { value: "user", label: "User (account-wide)" },
              { value: "agent", label: "Agent (per-agent default)" },
            ]}
            onChange={(v) =>
              setDraft((s) =>
                s
                  ? { ...s, preferred_scope: v as "session" | "user" | "agent" }
                  : s,
              )
            }
          />
        </Field>

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

