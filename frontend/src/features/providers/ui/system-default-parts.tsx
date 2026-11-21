import { AlertCircle, Loader2, RefreshCw } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useConfiguredModels, useProviderMetaField } from "@/platform/entities";
import { usePinnedAgents } from "../model/pinned-agents";

/** The live system default pair, in the words of whichever part is missing when there is none. */
export function LiveDefaultPair({
  loading,
  loadError,
  onRetry,
}: {
  loading: boolean;
  loadError: string | null;
  onRetry: () => void;
}) {
  const providerId = useProviderMetaField("default_id");
  const modelId = useProviderMetaField("default_model");

  if (providerId && modelId) {
    return (
      <div className="flex flex-col gap-1">
        <p
          data-testid="system-default-live-pair"
          className="m-0 break-all font-mono text-base font-semibold text-foreground"
        >
          {`${providerId}/${modelId}`}
        </p>
        {loadError && (
          <p role="alert" className="m-0 flex flex-wrap items-center gap-2 font-mono text-xs text-destructive">
            Could not refresh. This is the last known default.
            <Button type="button" variant="ghost" size="sm" onClick={onRetry} className="h-6 gap-1 px-2 text-xs">
              <RefreshCw size={11} aria-hidden="true" /> Retry
            </Button>
          </p>
        )}
        <DefaultAvailabilityNote providerId={providerId} modelId={modelId} />
      </div>
    );
  }

  if (loadError) {
    return (
      <div role="alert" className="flex flex-wrap items-center gap-2">
        <AlertCircle size={14} className="shrink-0 text-destructive" aria-hidden="true" />
        <p className="m-0 font-mono text-xs text-destructive">
          Could not load the system default. {loadError}
        </p>
        <Button type="button" variant="secondary" size="sm" onClick={onRetry} className="h-7 gap-1 text-xs">
          <RefreshCw size={12} aria-hidden="true" /> Retry
        </Button>
      </div>
    );
  }

  if (loading) {
    return (
      <p role="status" className="m-0 flex items-center gap-2 font-mono text-xs text-muted-foreground">
        <Loader2 size={12} className="animate-spin" aria-hidden="true" /> Loading the system default…
      </p>
    );
  }

  return (
    <p role="status" className="m-0 font-mono text-xs text-foreground">
      {providerId
        ? `${providerId} is the default provider but has no default model. Choose a model below.`
        : "No default provider is set. Choose a model below to set one."}
    </p>
  );
}

/**
 * Says so when the live pair is not among the enabled models of an enabled provider. The runtime
 * then cannot serve it and answers with the configured fallback model, when there is one.
 */
function DefaultAvailabilityNote({ providerId, modelId }: { providerId: string; modelId: string }) {
  const { items, isLoading } = useConfiguredModels();
  if (isLoading || items.length === 0) return null;
  const route = `${providerId}/${modelId}`;
  if (items.some((model) => model.id === route)) return null;
  return (
    <p className="m-0 font-body text-xs text-muted-foreground">
      This model is not currently enabled on its provider (disabled, removed or not verified). Until it is
      available again, the runtime answers with the configured fallback model, if one is set.
    </p>
  );
}

/** Agents that keep their own model, and how many follow the system default. */
export function PinnedAgentsList({ onRetry }: { onRetry: () => void }) {
  const snapshot = usePinnedAgents();

  if (!snapshot) {
    return (
      <p role="status" className="m-0 font-mono text-xs text-muted-foreground">
        Checking which agents keep their own model…
      </p>
    );
  }

  if (snapshot.status === "error") {
    return (
      <div role="alert" className="flex flex-wrap items-center gap-2">
        <p className="m-0 font-mono text-xs text-destructive">
          Pinned agents unknown: the agent list could not be loaded, so some agents may keep their own
          model.
        </p>
        <Button type="button" variant="secondary" size="sm" onClick={onRetry} className="h-7 gap-1 text-xs">
          <RefreshCw size={12} aria-hidden="true" /> Retry
        </Button>
      </div>
    );
  }

  const { pinned, follow_count: followCount } = snapshot;
  return (
    <div className="flex flex-col gap-1.5">
      <p className="m-0 font-mono text-[11px] uppercase tracking-widest text-muted-foreground">
        Agents with their own model ({pinned.length})
      </p>
      {pinned.length === 0 ? (
        <p className="m-0 font-body text-xs text-muted-foreground">No agent keeps its own model.</p>
      ) : (
        <ul className="m-0 flex max-h-32 list-none flex-col gap-1 overflow-y-auto p-0">
          {pinned.map((agent) => (
            <li
              key={agent.id}
              data-testid="system-default-pinned-agent"
              className="flex flex-wrap items-baseline gap-x-2 font-mono text-xs"
            >
              <span className="text-foreground">{agent.title}</span>
              <span className="break-all text-muted-foreground">{agent.model}</span>
            </li>
          ))}
        </ul>
      )}
      <p className="m-0 font-body text-xs text-muted-foreground">
        <span data-testid="system-default-follow-count" className="font-mono text-foreground">
          {followCount}
        </span>{" "}
        {followCount === 1 ? "other agent follows" : "other agents follow"} the system default.
      </p>
    </div>
  );
}
