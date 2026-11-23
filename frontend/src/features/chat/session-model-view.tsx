import { useEffect, useRef, type ReactNode } from "react";
import { AlertCircle, RefreshCw } from "lucide-react";

import { Button } from "@/components/ui/button";
import { modelOriginLabel } from "@/features/chat/model/model-receipt";
import { useAgentSession, useSessionEffectiveModelField } from "@/platform/entities";

/**
 * What a conversation is answered by, and what clearing its own model would give it.
 * Everything here is the runtime's answer (effective-config), shown for the last saved settings;
 * an unsaved or refused choice never changes it.
 */

function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex flex-wrap items-baseline gap-x-3 gap-y-0.5">
      <dt className="w-20 shrink-0 font-mono text-[11px] text-muted-foreground">{label}</dt>
      <dd className="m-0 min-w-0 font-mono text-xs text-foreground">{children}</dd>
    </div>
  );
}

function RequestedModel({ sessionId }: { sessionId: string }) {
  const requested = useAgentSession(sessionId)?.model ?? null;
  return requested ? (
    <span data-testid="session-config-requested" className="break-all">
      {requested}
    </span>
  ) : (
    <span className="text-muted-foreground">none (inherits)</span>
  );
}

function EffectiveModel({ sessionId }: { sessionId: string }) {
  const model = useSessionEffectiveModelField(sessionId, "model");
  const origin = useSessionEffectiveModelField(sessionId, "origin");
  const reason = useSessionEffectiveModelField(sessionId, "unavailable_reason");
  if (model) {
    return (
      <>
        <span data-testid="session-config-effective-model" className="break-all">
          {model}
        </span>
        {origin && (
          <>
            <span className="text-muted-foreground"> · </span>
            <span data-testid="session-config-effective-origin" className="text-muted-foreground">
              {modelOriginLabel(origin)}
            </span>
          </>
        )}
      </>
    );
  }
  return (
    <span className="text-destructive">
      No model can answer right now{reason ? `: ${reason}` : "."}
    </span>
  );
}

export function SessionEffectiveModelView({
  sessionId,
  failed,
  onRetry,
}: {
  sessionId: string;
  /** The effective-config request failed; nothing can be said about the answering model. */
  failed: boolean;
  onRetry: () => void;
}) {
  const stored = useSessionEffectiveModelField(sessionId, "stored");

  if (failed) {
    return (
      <div
        data-testid="session-config-effective"
        role="alert"
        className="flex items-start justify-between gap-3 rounded-lg bg-destructive/10 p-3"
      >
        <div className="flex items-start gap-2">
          <AlertCircle size={14} className="mt-0.5 shrink-0 text-destructive" aria-hidden="true" />
          <p className="m-0 font-mono text-xs text-destructive">
            Could not check which model answers this conversation.
          </p>
        </div>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          onClick={onRetry}
          className="shrink-0 gap-1.5 font-mono text-xs"
        >
          <RefreshCw size={12} aria-hidden="true" /> Retry
        </Button>
      </div>
    );
  }

  return (
    <section
      data-testid="session-config-effective"
      aria-label="Model answering this conversation"
      aria-busy={stored === undefined}
      className="flex flex-col gap-2 rounded-lg bg-muted/40 p-3"
    >
      {stored === undefined ? (
        <p role="status" className="m-0 font-mono text-xs text-muted-foreground">
          Checking which model answers…
        </p>
      ) : (
        <dl className="m-0 flex flex-col gap-1.5">
          <Row label="Requested">
            <RequestedModel sessionId={sessionId} />
          </Row>
          <Row label="Effective">
            <EffectiveModel sessionId={sessionId} />
          </Row>
        </dl>
      )}
      {stored === false && (
        <p className="m-0 font-body text-xs text-muted-foreground">
          Nothing is saved for this conversation yet, so it follows its agent, then the system default.
        </p>
      )}
      <p className="m-0 font-body text-xs text-muted-foreground">
        Shows the last saved settings. A change counts only after you save and the runtime accepts it.
      </p>
    </section>
  );
}

function InheritedModel({ sessionId }: { sessionId: string }) {
  const model = useSessionEffectiveModelField(sessionId, "inherited_model");
  const origin = useSessionEffectiveModelField(sessionId, "inherited_origin");
  const reason = useSessionEffectiveModelField(sessionId, "inherited_unavailable_reason");
  if (model) {
    return (
      <p className="m-0 font-mono text-xs text-foreground">
        If you clear this override, this conversation will answer with{" "}
        <span data-testid="session-config-inherited-model" className="break-all font-medium">
          {model}
        </span>
        {origin ? ` from ${modelOriginLabel(origin)}.` : "."}
      </p>
    );
  }
  return (
    <p role="alert" className="m-0 font-mono text-xs text-destructive">
      Clearing this override would leave the conversation without a model that can answer
      {reason ? `: ${reason}` : "."} Keep the override, or fix the default first.
    </p>
  );
}

/** Previews the model a conversation would inherit before its override is cleared. Cancel changes nothing. */
export function ClearOverridePreview({
  sessionId,
  failed,
  busy,
  onConfirm,
  onCancel,
}: {
  sessionId: string;
  failed: boolean;
  busy: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const stored = useSessionEffectiveModelField(sessionId, "stored");
  const inherited = useSessionEffectiveModelField(sessionId, "inherited_model");
  const known = !failed && stored === true;
  const unsaved = !failed && stored === false;
  const checking = !failed && stored === undefined;
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    ref.current?.focus();
  }, []);

  return (
    <div
      ref={ref}
      tabIndex={-1}
      role="group"
      aria-label="Clear model override"
      data-testid="session-config-clear-preview"
      className="flex flex-col gap-3 rounded-lg bg-muted/40 p-3 outline-none focus-visible:ring-3 focus-visible:ring-ring"
    >
      {known ? (
        <InheritedModel sessionId={sessionId} />
      ) : unsaved ? (
        <p className="m-0 font-mono text-xs text-foreground">
          This override is not saved yet. Clearing it discards the choice.
        </p>
      ) : checking ? (
        <p role="status" className="m-0 font-mono text-xs text-muted-foreground">
          Checking what this conversation would inherit…
        </p>
      ) : (
        <p role="alert" className="m-0 font-mono text-xs text-destructive">
          Could not check what this conversation would inherit. Keep the override and try again.
        </p>
      )}
      <div className="flex gap-2">
        <Button
          type="button"
          size="sm"
          data-testid="session-config-clear-confirm"
          disabled={busy || !(unsaved || (known && inherited))}
          onClick={onConfirm}
          className="font-mono text-xs"
        >
          Confirm clear
        </Button>
        <Button
          type="button"
          size="sm"
          variant="secondary"
          data-testid="session-config-clear-cancel"
          disabled={busy}
          onClick={onCancel}
          className="font-mono text-xs"
        >
          Cancel
        </Button>
      </div>
    </div>
  );
}
