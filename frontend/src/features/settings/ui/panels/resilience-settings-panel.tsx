import { useCallback, useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useSettings } from "../../model/use-settings";
import { SettingsHint } from "../generic-schema-panel";
import {
  AdvancedSection,
  ErrorBanner,
  Field,
  PanelHeader,
  SavedBanner,
  SettingSelect,
  Toggle,
} from "../settings-primitives";
import { RESILIENCE_RECOMMENDED_DEFAULTS } from "./resilience-preview";

export function ResiliencePanel() {
  const { values, loading, saving, error, setSetting, saveAll, reload } =
    useSettings("resilience");
  const [savedFlash, setSavedFlash] = useState(false);
  const [statusListInput, setStatusListInput] = useState("");

  const valueFor = (key: string): unknown => values[`resilience.${key}`];
  const setField = (key: string, value: unknown) =>
    setSetting(`resilience.${key}`, value);

  const retryableHttpStatuses = values["resilience.retryable_http_statuses"];

  useEffect(() => {
    if (!Array.isArray(retryableHttpStatuses)) return;
    const text = retryableHttpStatuses.join(", ");
    queueMicrotask(() => {
      setStatusListInput(text);
    });
  }, [retryableHttpStatuses]);

  const parsedStatuses = statusListInput
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
  const hasInvalidStatusToken = parsedStatuses.some((item) => {
    const n = Number(item);
    return !Number.isInteger(n) || n < 100 || n > 599;
  });
  const statusNumbers = parsedStatuses
    .map((item) => Number(item))
    .filter((n) => Number.isInteger(n) && n >= 100 && n <= 599);

  const validationErrors: Record<string, string> = {};
  const reqPerSec = Number(valueFor("requests_per_second"));
  if (!Number.isFinite(reqPerSec) || reqPerSec < 0.1) {
    validationErrors.requests_per_second = "Must be at least 0.1 requests/sec.";
  }
  const burstSize = Number(valueFor("burst_size"));
  if (!Number.isFinite(burstSize) || burstSize < 1) {
    validationErrors.burst_size = "Burst size must be at least 1.";
  }
  const requestTimeoutMs = Number(valueFor("request_timeout_ms"));
  if (!Number.isFinite(requestTimeoutMs) || requestTimeoutMs < 1000) {
    validationErrors.request_timeout_ms =
      "Request timeout must be at least 1000ms.";
  }
  const streamStartTimeoutMs = Number(valueFor("stream_start_timeout_ms"));
  if (!Number.isFinite(streamStartTimeoutMs) || streamStartTimeoutMs < 1000) {
    validationErrors.stream_start_timeout_ms =
      "Stream start timeout must be at least 1000ms.";
  }
  const retryMaxAttempts = Number(valueFor("retry_max_attempts"));
  if (
    !Number.isFinite(retryMaxAttempts) ||
    retryMaxAttempts < 0 ||
    retryMaxAttempts > 10
  ) {
    validationErrors.retry_max_attempts =
      "Retry attempts must be between 0 and 10.";
  }
  const retryBaseDelayMs = Number(valueFor("retry_base_delay_ms"));
  if (!Number.isFinite(retryBaseDelayMs) || retryBaseDelayMs < 100) {
    validationErrors.retry_base_delay_ms = "Base delay must be at least 100ms.";
  }
  const retryBackoffMultiplier = Number(valueFor("retry_backoff_multiplier"));
  if (
    !Number.isFinite(retryBackoffMultiplier) ||
    retryBackoffMultiplier < 1.1 ||
    retryBackoffMultiplier > 5.0
  ) {
    validationErrors.retry_backoff_multiplier =
      "Backoff multiplier must be between 1.1 and 5.0.";
  }
  const retryMaxDelayMs = Number(valueFor("retry_max_delay_ms"));
  if (!Number.isFinite(retryMaxDelayMs) || retryMaxDelayMs < 100) {
    validationErrors.retry_max_delay_ms = "Max delay must be at least 100ms.";
  }
  const retryBudgetMs = Number(valueFor("retry_budget_ms"));
  if (!Number.isFinite(retryBudgetMs) || retryBudgetMs < 0) {
    validationErrors.retry_budget_ms = "Retry budget must be 0 or greater.";
  }
  if (hasInvalidStatusToken || statusNumbers.length === 0) {
    validationErrors.retryable_http_statuses =
      "Use comma-separated HTTP codes between 100 and 599.";
  }

  const hasErrors = Object.keys(validationErrors).length > 0;

  const handleSave = useCallback(async () => {
    if (hasErrors) return;
    await saveAll();
    setSavedFlash(true);
    setTimeout(() => setSavedFlash(false), 2500);
  }, [hasErrors, saveAll]);

  const resetRecommendedDefaults = useCallback(() => {
    Object.entries(RESILIENCE_RECOMMENDED_DEFAULTS).forEach(([key, value]) => {
      setSetting(`resilience.${key}`, value);
    });
    setStatusListInput(
      RESILIENCE_RECOMMENDED_DEFAULTS.retryable_http_statuses.join(", "),
    );
  }, [setSetting]);

  const renderError = (key: string) =>
    validationErrors[key] ? (
      <p className="font-mono text-xs text-destructive">
        {validationErrors[key]}
      </p>
    ) : null;

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <PanelHeader
        title="Resilience"
        subtitle="Configure global rate limits, timeout budgets, and retry behavior."
        saving={saving}
        loading={loading}
        saveDisabled={hasErrors}
        onSave={() => void handleSave()}
        onReload={() => void reload()}
      />
      <div className="flex-1 overflow-y-auto px-6 py-5 space-y-6">
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
        <SettingsHint id="resilience">
          Resilience settings protect your runtime from overload and API
          failures. Rate limiting prevents too many requests, timeouts stop hung
          requests, and retries automatically recover from transient errors. The
          defaults work well for most deployments — only adjust if you're seeing
          specific issues.
        </SettingsHint>
        {!loading && (
          <>
            <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
              <div>
                <p className="font-mono text-xs font-medium text-foreground">
                  Recommended defaults
                </p>
                <p className="font-mono text-xs text-muted-foreground">
                  Reset all resilience controls to production-safe defaults.
                </p>
              </div>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={resetRecommendedDefaults}
              >
                Reset Defaults
              </Button>
            </div>

            <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground/50">
              Rate limiting
            </p>
            <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
              <div>
                <p className="font-mono text-xs font-medium text-foreground">
                  Rate Limiting
                </p>
                <p className="font-mono text-xs text-muted-foreground">
                  Throttle inbound API traffic to reduce overload bursts.
                </p>
              </div>
              <Toggle
                value={(valueFor("rate_limit_enabled") as boolean) ?? true}
                onChange={(v) => setField("rate_limit_enabled", v)}
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <Field
                label="Requests Per Second"
                hint="Lower value = stricter global throttling."
                defaultValue="10"
              >
                <Input
                  type="number"
                  step={0.1}
                  min={0.1}
                  value={(valueFor("requests_per_second") as number) ?? ""}
                  onChange={(e) =>
                    setField("requests_per_second", Number(e.target.value))
                  }
                  className="font-mono text-xs"
                />
                {renderError("requests_per_second")}
              </Field>
              <Field
                label="Burst Size"
                hint="How many requests can pass in a short spike."
                defaultValue="20"
              >
                <Input
                  type="number"
                  min={1}
                  value={(valueFor("burst_size") as number) ?? ""}
                  onChange={(e) =>
                    setField("burst_size", Number(e.target.value))
                  }
                  className="font-mono text-xs"
                />
                {renderError("burst_size")}
              </Field>
            </div>

            <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground/50">
              Timeouts
            </p>
            <div className="grid grid-cols-2 gap-4">
              <Field
                label="Request Timeout (ms)"
                hint="Upper bound for non-stream response wait time."
                defaultValue="30000"
              >
                <Input
                  type="number"
                  min={1000}
                  value={(valueFor("request_timeout_ms") as number) ?? ""}
                  onChange={(e) =>
                    setField("request_timeout_ms", Number(e.target.value))
                  }
                  className="font-mono text-xs"
                />
                {renderError("request_timeout_ms")}
              </Field>
              <Field
                label="Stream Start Timeout (ms)"
                hint="How long to wait for first stream chunk before retry/fail."
                defaultValue="15000"
              >
                <Input
                  type="number"
                  min={1000}
                  value={(valueFor("stream_start_timeout_ms") as number) ?? ""}
                  onChange={(e) =>
                    setField("stream_start_timeout_ms", Number(e.target.value))
                  }
                  className="font-mono text-xs"
                />
                {renderError("stream_start_timeout_ms")}
              </Field>
            </div>

            <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground/50">
              Retries
            </p>
            <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
              <div>
                <p className="font-mono text-xs font-medium text-foreground">
                  Retries Enabled
                </p>
                <p className="font-mono text-xs text-muted-foreground">
                  Retry transient failures with exponential backoff.
                </p>
              </div>
              <Toggle
                value={(valueFor("retries_enabled") as boolean) ?? true}
                onChange={(v) => setField("retries_enabled", v)}
              />
            </div>
            <div className="grid grid-cols-3 gap-4">
              <Field
                label="Max Attempts"
                hint="Total attempts including the first request."
                defaultValue="3"
              >
                <Input
                  type="number"
                  min={0}
                  max={10}
                  value={(valueFor("retry_max_attempts") as number) ?? ""}
                  onChange={(e) =>
                    setField("retry_max_attempts", Number(e.target.value))
                  }
                  className="font-mono text-xs"
                />
                {renderError("retry_max_attempts")}
              </Field>
              <Field
                label="Base Delay (ms)"
                hint="Initial retry delay before backoff multiplier."
                defaultValue="1000"
              >
                <Input
                  type="number"
                  min={100}
                  value={(valueFor("retry_base_delay_ms") as number) ?? ""}
                  onChange={(e) =>
                    setField("retry_base_delay_ms", Number(e.target.value))
                  }
                  className="font-mono text-xs"
                />
                {renderError("retry_base_delay_ms")}
              </Field>
              <Field
                label="Max Delay (ms)"
                hint="Hard cap for retry wait interval."
                defaultValue="10000"
              >
                <Input
                  type="number"
                  min={100}
                  value={(valueFor("retry_max_delay_ms") as number) ?? ""}
                  onChange={(e) =>
                    setField("retry_max_delay_ms", Number(e.target.value))
                  }
                  className="font-mono text-xs"
                />
                {renderError("retry_max_delay_ms")}
              </Field>
            </div>

            <AdvancedSection label="Backoff, jitter & retry budget">
              <div className="grid grid-cols-2 gap-4">
                <Field
                  label="Backoff Multiplier"
                  hint="Growth factor for each retry delay step."
                  defaultValue="2"
                >
                  <Input
                    type="number"
                    step={0.1}
                    min={1.1}
                    max={5}
                    value={
                      (valueFor("retry_backoff_multiplier") as number) ?? ""
                    }
                    onChange={(e) =>
                      setField(
                        "retry_backoff_multiplier",
                        Number(e.target.value),
                      )
                    }
                    className="font-mono text-xs"
                  />
                  {renderError("retry_backoff_multiplier")}
                </Field>
                <Field
                  label="Retry Budget (ms)"
                  hint="Maximum total waiting time spent on retries."
                  defaultValue="20000"
                >
                  <Input
                    type="number"
                    min={0}
                    value={(valueFor("retry_budget_ms") as number) ?? ""}
                    onChange={(e) =>
                      setField("retry_budget_ms", Number(e.target.value))
                    }
                    className="font-mono text-xs"
                  />
                  {renderError("retry_budget_ms")}
                </Field>
                <Field
                  label="Jitter Mode"
                  hint="Spreads retries to avoid synchronized spikes."
                  defaultValue="full"
                >
                  <SettingSelect
                    value={(valueFor("retry_jitter_mode") as string) ?? "full"}
                    options={[
                      { value: "none", label: "None" },
                      { value: "full", label: "Full" },
                      { value: "equal", label: "Equal" },
                      { value: "decorrelated", label: "Decorrelated" },
                    ]}
                    onChange={(v) => setField("retry_jitter_mode", v)}
                  />
                </Field>
                <div className="space-y-3">
                  <div className="flex items-center justify-between rounded-md border border-border bg-muted/20 px-3 py-2">
                    <span className="font-mono text-xs text-foreground">
                      Respect Retry-After
                    </span>
                    <Toggle
                      value={
                        (valueFor("retry_respect_retry_after") as boolean) ??
                        true
                      }
                      onChange={(v) => setField("retry_respect_retry_after", v)}
                    />
                  </div>
                  <div className="flex items-center justify-between rounded-md border border-border bg-muted/20 px-3 py-2">
                    <span className="font-mono text-xs text-foreground">
                      Retry Transport Errors
                    </span>
                    <Toggle
                      value={
                        (valueFor("retryable_transport_errors") as boolean) ??
                        true
                      }
                      onChange={(v) =>
                        setField("retryable_transport_errors", v)
                      }
                    />
                  </div>
                </div>
              </div>
              <Field
                label="Retryable HTTP Status Codes"
                hint="Comma-separated status codes (100-599)."
                defaultValue="408, 425, 429, 500, 502, 503, 504"
              >
                <Input
                  value={statusListInput}
                  onChange={(e) => {
                    const raw = e.target.value;
                    setStatusListInput(raw);
                    const next = raw
                      .split(",")
                      .map((item) => Number(item.trim()))
                      .filter(
                        (n) => Number.isInteger(n) && n >= 100 && n <= 599,
                      );
                    if (next.length > 0)
                      setField("retryable_http_statuses", next);
                  }}
                  placeholder="408, 425, 429, 500, 502, 503, 504"
                  className="font-mono text-xs"
                />
                {renderError("retryable_http_statuses")}
              </Field>
            </AdvancedSection>
          </>
        )}
      </div>
    </div>
  );
}

