import { useCallback, useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { useLocation } from "react-router";

import { Button } from "@/components/ui/button";
import { CatalogModelPicker } from "@/features/models/pickers";
import {
  SystemDefaultModelError,
  setSystemDefaultModel,
  useProviderMetaField,
} from "@/platform/entities";
import { loadPinnedAgents } from "../model/pinned-agents";
import { useSystemDefaultData } from "../model/use-system-default-data";
import { LiveDefaultPair, PinnedAgentsList } from "./system-default-parts";

/** URL hash that opens the panel with its picker focused (command palette, shared links). */
export const SYSTEM_DEFAULT_PANEL_HASH = "#system-default-model";
const PICKER_ID = "system-default-model-picker";
const STATUS_ID = "system-default-status";

/** `provider/model` to its two parts. The provider id has no `/`; the model id may. */
function splitRoute(route: string): { providerId: string; modelId: string } | null {
  const slash = route.indexOf("/");
  if (slash <= 0 || slash === route.length - 1) return null;
  return { providerId: route.slice(0, slash), modelId: route.slice(slash + 1) };
}

function errorMessage(error: unknown): string {
  if (error instanceof SystemDefaultModelError) return error.message;
  return `System default not changed. ${error instanceof Error ? error.message : "The runtime did not answer."}`;
}

/**
 * Changes the system default model: the model every conversation, agent and team uses unless it
 * names its own. Only models of enabled providers can be chosen. The change is shown as done only
 * after the runtime acknowledged it, with the pair that was requested and the word `live`.
 */
export function SystemDefaultPanel() {
  const data = useSystemDefaultData();
  const [selection, setSelection] = useState("");
  const [saving, setSaving] = useState(false);
  const [requested, setRequested] = useState<string | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const livePair = useLivePairRoute();
  const target = splitRoute(selection);
  const unchanged = selection !== "" && selection === livePair;
  const canConfirm = !saving && target !== null && !unchanged;

  useEffect(() => {
    void loadPinnedAgents();
  }, []);
  useFocusOnHash();

  const confirm = useCallback(async () => {
    const parts = splitRoute(selection);
    if (!parts) return;
    setSaving(true);
    setRequested(null);
    setFailure(null);
    try {
      await setSystemDefaultModel(parts.providerId, parts.modelId);
      setRequested(selection);
      setSelection("");
      void loadPinnedAgents();
    } catch (error) {
      setFailure(errorMessage(error));
    } finally {
      setSaving(false);
    }
  }, [selection]);

  return (
    <section
      aria-labelledby="system-default-heading"
      className="flex flex-col gap-3 bg-card px-4 py-4 sm:px-6"
    >
      <div className="flex flex-col gap-1">
        <h2
          id="system-default-heading"
          className="m-0 font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground"
        >
          System default model
        </h2>
        <p className="m-0 font-body text-xs text-muted-foreground">
          Answers every conversation, agent and team that does not name its own model. It changes as soon as the
          runtime confirms it, with no restart.
        </p>
      </div>

      <LiveDefaultPair
        loading={data.registryLoading}
        loadError={data.registryError}
        onRetry={data.retry}
      />

      <div className="flex flex-col gap-2 sm:flex-row sm:items-start">
        <div className="min-w-0 flex-1">
          <CatalogModelPicker
            id={PICKER_ID}
            testId="system-default-picker-trigger"
            ariaLabel="System default model"
            ariaDescribedBy={STATUS_ID}
            ariaInvalid={failure !== null}
            placeholder="Choose the new system default model"
            value={selection}
            onChange={(route) => {
              setSelection(route);
              setFailure(null);
              setRequested(null);
            }}
            disabled={saving}
          />
        </div>
        <Button
          type="button"
          data-testid="system-default-confirm"
          disabled={!canConfirm}
          onClick={() => void confirm()}
          className="shrink-0 gap-1.5 font-mono text-xs"
        >
          {saving && <Loader2 size={12} className="animate-spin" aria-hidden="true" />}
          {saving ? "Waiting for the runtime…" : "Set as system default"}
        </Button>
      </div>

      <div id={STATUS_ID} className="flex flex-col gap-2" aria-live="polite">
        {unchanged && (
          <p className="m-0 font-mono text-xs text-muted-foreground">That model is already the system default.</p>
        )}
        {data.catalogError && (
          <p role="alert" className="m-0 font-mono text-xs text-destructive">
            The model list could not be loaded: {data.catalogError}
          </p>
        )}
        {failure && (
          <p
            role="alert"
            data-testid="system-default-error"
            className="m-0 rounded-lg bg-destructive/10 p-3 font-mono text-xs text-destructive"
          >
            {failure}
          </p>
        )}
        {requested && (
          <div
            role="status"
            data-testid="system-default-confirmation"
            className="flex flex-col gap-1 rounded-lg bg-success/10 p-3 font-mono text-xs"
          >
            <p className="m-0 font-medium text-foreground">System default changed</p>
            <p className="m-0 flex flex-wrap gap-x-3">
              <span className="text-muted-foreground">Requested</span>
              <span className="break-all text-foreground">{requested}</span>
            </p>
            <p className="m-0 flex flex-wrap gap-x-3">
              <span className="text-muted-foreground">Activation</span>
              <span className="text-foreground">live (no restart)</span>
            </p>
          </div>
        )}
      </div>

      <PinnedAgentsList onRetry={() => void loadPinnedAgents()} />
    </section>
  );
}

/** The live `provider/model` pair as one comparable string; empty until both parts exist. */
function useLivePairRoute(): string {
  const providerId = useProviderMetaField("default_id");
  const modelId = useProviderMetaField("default_model");
  return providerId && modelId ? `${providerId}/${modelId}` : "";
}

/** Moves focus to the picker when the page is opened at the panel's hash, so a command lands on it. */
function useFocusOnHash() {
  const location = useLocation();
  useEffect(() => {
    if (location.hash !== SYSTEM_DEFAULT_PANEL_HASH) return;
    const frame = requestAnimationFrame(() => {
      const trigger = document.getElementById(PICKER_ID);
      trigger?.scrollIntoView({ block: "nearest" });
      trigger?.focus();
    });
    return () => cancelAnimationFrame(frame);
  }, [location.hash, location.key]);
}
