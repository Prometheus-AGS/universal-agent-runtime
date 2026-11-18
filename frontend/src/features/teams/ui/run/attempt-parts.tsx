import { AlertTriangle } from "lucide-react";
import { useAttemptField } from "@/platform/entities";
import { formatCount, formatDateTime, formatMicrocost, formatSeconds } from "../../model/format";
import { Disclosure } from "./disclosure";
import { humanizeToken, isChargedInFull } from "./run-model";
import { useTeamCurrency, type RunContext } from "./run-hooks";

interface AttemptRef extends RunContext {
  attemptId: string;
}

/** `providerId/modelId` of the route the runtime resolved for the attempt. */
export function ModelLine({ workspace, teamId, attemptId }: AttemptRef) {
  const models = useAttemptField(workspace, teamId, attemptId, "effectiveModels");
  const first = models?.[0]?.route;
  if (!first) return <p className="text-xs text-muted-foreground">Model: not resolved yet</p>;
  const extra = (models?.length ?? 1) - 1;
  const priced = models?.[0]?.pricingIdentity;
  const pricedElsewhere = priced !== undefined && (priced.providerId !== first.providerId || priced.modelId !== first.modelId);
  return (
    <div className="min-w-0 space-y-0.5 text-xs break-all text-muted-foreground">
      <p>
        Model: <span className="font-mono">{first.providerId}/{first.modelId}</span>
        {extra > 0 ? ` and ${extra} more` : ""}
      </p>
      {pricedElsewhere && (
        <p>
          Billed at the rate of <span className="font-mono">{priced.providerId}/{priced.modelId}</span>
        </p>
      )}
    </div>
  );
}

/**
 * Settled usage when the runtime has it, otherwise the reservation. A settled attempt without a usage report
 * keeps its whole reservation against the team budget (`runtime/team_execution/execution.rs`), so it says so in
 * the warning treatment: a bar, a tint and an icon, never colour alone.
 */
export function UsageLine({ workspace, teamId, attemptId }: AttemptRef) {
  const usage = useAttemptField(workspace, teamId, attemptId, "usage");
  const reservation = useAttemptField(workspace, teamId, attemptId, "reservation");
  const status = useAttemptField(workspace, teamId, attemptId, "status");
  const currency = useTeamCurrency(workspace, teamId);
  const figures = usage ?? reservation;
  if (!figures) return null;
  const figureText = `${formatCount(figures.tokens)} tokens, ${formatMicrocost(figures.costMicrounits, currency)}, ${formatSeconds(figures.elapsedSeconds)}`;
  if (isChargedInFull(status, usage)) {
    return (
      <p className="flex items-start gap-1.5 rounded-md bg-warning/15 px-2 py-1.5 text-sm break-words text-foreground">
        <AlertTriangle aria-hidden="true" className="mt-0.5 size-4 shrink-0" />
        <span>
          Charged in full, no usage report: {figureText}
        </span>
      </p>
    );
  }
  const label = usage ? "Used" : "Reserved (usage not settled)";
  return (
    <p className="text-xs text-muted-foreground">
      {label}: {figureText}
    </p>
  );
}

export function TimeLine({ workspace, teamId, attemptId }: AttemptRef) {
  const createdAt = useAttemptField(workspace, teamId, attemptId, "createdAt");
  const updatedAt = useAttemptField(workspace, teamId, attemptId, "updatedAt");
  return (
    <p className="text-xs text-muted-foreground">
      Created {formatDateTime(createdAt)}
      {updatedAt && updatedAt !== createdAt ? `, updated ${formatDateTime(updatedAt)}` : ""}
    </p>
  );
}

const SHORT_REASON_LIMIT = 160;

/** Plain words for runtime reason codes whose meaning is confirmed in the runtime source. */
const REASON_TEXT: Readonly<Record<string, string>> = {
  team_usage_or_effect_outcome_uncertain:
    "The runtime did not receive a usage report for a model call, so the whole reservation counts against the team budget.",
};

/**
 * The runtime's structured diagnostic when there is one (code, category, stage, suggested action), then the
 * state reason. A reason that can embed JSON or runs long sits behind an expander; a short plain token such
 * as `output_budget_unavailable` is shown as it is.
 */
export function DiagnosticBlock({ workspace, teamId, attemptId }: AttemptRef) {
  const diagnostic = useAttemptField(workspace, teamId, attemptId, "diagnostic");
  const reason = useAttemptField(workspace, teamId, attemptId, "stateReason");
  const longReason = reason !== null && reason !== undefined && (reason.length > SHORT_REASON_LIMIT || reason.includes("{"));
  if (!diagnostic && !reason) return null;
  const explanation = reason ? REASON_TEXT[reason] : undefined;
  return (
    <div className="min-w-0 space-y-1.5">
      {diagnostic && (
        <dl className="grid gap-x-4 gap-y-1 text-sm sm:grid-cols-[max-content_minmax(0,1fr)]">
          <dt className="text-xs text-muted-foreground">Diagnostic</dt>
          <dd className="font-mono text-xs break-all">{diagnostic.code}</dd>
          {diagnostic.category && (
            <>
              <dt className="text-xs text-muted-foreground">Category</dt>
              <dd className="break-words">{humanizeToken(diagnostic.category)}</dd>
            </>
          )}
          {diagnostic.sourceStage && (
            <>
              <dt className="text-xs text-muted-foreground">Stage</dt>
              <dd className="break-words">{humanizeToken(diagnostic.sourceStage)}</dd>
            </>
          )}
          <dt className="text-xs text-muted-foreground">Suggested action</dt>
          <dd className="break-words">
            {humanizeToken(diagnostic.action)}
            <span className="text-xs text-muted-foreground"> ({diagnostic.retryable ? "can be retried" : "retrying will not help"})</span>
          </dd>
          {diagnostic.protectedDiagnosticRef && (
            <>
              <dt className="text-xs text-muted-foreground">Reference</dt>
              <dd className="font-mono text-xs break-all">{diagnostic.protectedDiagnosticRef}</dd>
            </>
          )}
        </dl>
      )}
      {reason && (explanation ? (
        <>
          <p className="text-sm break-words">{explanation}</p>
          <Disclosure label="Runtime detail">
            <p className="rounded-md bg-background/70 p-2 font-mono text-xs break-all whitespace-pre-wrap">{reason}</p>
          </Disclosure>
        </>
      ) : diagnostic || longReason ? (
        <Disclosure label="Runtime detail">
          <p className="rounded-md bg-background/70 p-2 font-mono text-xs break-all whitespace-pre-wrap">{reason}</p>
        </Disclosure>
      ) : (
        <p className="text-sm break-words">
          <span className="text-xs text-muted-foreground">Reason: </span>
          <span className="font-mono text-xs">{reason}</span>
        </p>
      ))}
    </div>
  );
}
