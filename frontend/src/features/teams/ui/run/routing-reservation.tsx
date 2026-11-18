import { useId } from "react";
import { Input } from "@/components/ui/input";
import { useTeamRunSummaryField } from "@/platform/entities";
import { formatCount, formatMicrocost, formatSeconds } from "../../model/format";
import { useRemaining, useTeamCurrency, type RunContext } from "./run-hooks";
import { remainingAfter, reservationHints, type ReservationDraft, type ReservationValue } from "./run-model";

/** Remaining budget and task slots, shown beside the reservation inputs. */
export function RemainingLine({ workspace, teamId }: RunContext) {
  const loadedAt = useTeamRunSummaryField(workspace, teamId, "loadedAt");
  const remaining = useRemaining({ workspace, teamId });
  const currency = useTeamCurrency(workspace, teamId);
  if (loadedAt === undefined) {
    return <p className="text-xs text-muted-foreground">Remaining budget appears once the execution summary has loaded.</p>;
  }
  return (
    <dl className="grid gap-x-6 gap-y-1 text-sm sm:grid-cols-2">
      <div>
        <dt className="text-xs text-muted-foreground">Budget left</dt>
        <dd className="break-words">
          {remaining.tokens === null ? "unknown" : `${formatCount(remaining.tokens)} tokens`},{" "}
          {remaining.costMicrounits === null ? "unknown" : formatMicrocost(remaining.costMicrounits, currency)},{" "}
          {remaining.elapsedSeconds === null ? "unknown" : formatSeconds(remaining.elapsedSeconds)}
        </dd>
      </div>
      <div>
        <dt className="text-xs text-muted-foreground">Task slots left</dt>
        <dd>
          {remaining.slots === null ? "unknown" : formatCount(remaining.slots)}
          <span className="text-xs text-muted-foreground"> (each test uses one, and none can be given back)</span>
        </dd>
      </div>
    </dl>
  );
}

interface ReservationFieldsProps {
  draft: ReservationDraft;
  currency: string | undefined;
  disabled: boolean;
  onChange: (next: ReservationDraft) => void;
}

/** One reservation input with the value it was read as, in units, beside it. The hint is empty while the text is not a valid value. */
function ReservationField({
  id,
  label,
  inputMode,
  value,
  hint,
  disabled,
  onChange,
}: {
  id: string;
  label: string;
  inputMode: "numeric" | "decimal";
  value: string;
  hint: string | null;
  disabled: boolean;
  onChange: (value: string) => void;
}) {
  return (
    <div className="min-w-0 space-y-1">
      <label htmlFor={id} className="text-sm font-medium">{label}</label>
      <Input id={id} inputMode={inputMode} value={value} disabled={disabled} aria-describedby={`${id}-hint`} onChange={(event) => onChange(event.target.value)} />
      <p id={`${id}-hint`} className="min-h-4 text-xs text-muted-foreground">{hint}</p>
    </div>
  );
}

/** The three reservation inputs. Cost is entered in currency units; the wire carries microunits. Each shows what it reads as, with units. */
export function ReservationFields({ draft, currency, disabled, onChange }: ReservationFieldsProps) {
  const base = useId();
  const hints = reservationHints(draft);
  return (
    <div className="grid gap-3 sm:grid-cols-3">
      <ReservationField
        id={`${base}-tokens`}
        label="Tokens"
        inputMode="numeric"
        value={draft.tokens}
        hint={hints.tokens === null ? null : `${formatCount(hints.tokens)} tokens`}
        disabled={disabled}
        onChange={(tokens) => onChange({ ...draft, tokens })}
      />
      <ReservationField
        id={`${base}-cost`}
        label={`Cost (${currency ?? "currency units"})`}
        inputMode="decimal"
        value={draft.cost}
        hint={hints.costMicrounits === null ? null : formatMicrocost(hints.costMicrounits, currency)}
        disabled={disabled}
        onChange={(cost) => onChange({ ...draft, cost })}
      />
      <ReservationField
        id={`${base}-seconds`}
        label="Elapsed seconds"
        inputMode="numeric"
        value={draft.elapsedSeconds}
        hint={hints.elapsedSeconds === null ? null : formatSeconds(hints.elapsedSeconds)}
        disabled={disabled}
        onChange={(elapsedSeconds) => onChange({ ...draft, elapsedSeconds })}
      />
    </div>
  );
}

/**
 * The cost of the test before it is sent, in the entered figures against what the team has left: what is held while
 * it runs, the one task slot it uses for good, and what remains once it is admitted. Shown only for a valid reservation.
 */
export function ConsequenceLine({ workspace, teamId, value }: RunContext & { value: ReservationValue | null }) {
  const remaining = useRemaining({ workspace, teamId });
  const currency = useTeamCurrency(workspace, teamId);
  if (value === null) return null;
  const after = remainingAfter(value, remaining);
  const slots = remaining.slots === null ? "a task slot" : `1 of the ${formatCount(remaining.slots)} task slots left`;
  const left = [
    after.slots === null ? null : `${formatCount(after.slots)} task ${after.slots === 1 ? "slot" : "slots"}`,
    after.costMicrounits === null ? null : formatMicrocost(after.costMicrounits, currency),
    after.tokens === null ? null : `${formatCount(after.tokens)} tokens`,
    after.elapsedSeconds === null ? null : formatSeconds(after.elapsedSeconds),
  ].filter((part): part is string => part !== null);
  return (
    <p className="max-w-prose rounded-md bg-background/70 px-3 py-2 text-sm break-words">
      Reserves {formatCount(value.tokens)} tokens, {formatMicrocost(value.costMicrounits, currency)} and{" "}
      {formatSeconds(value.elapsedSeconds)} while it runs, and uses {slots}, permanently.
      {left.length > 0 ? ` Left after admission: ${left.join(", ")}.` : ""}
    </p>
  );
}
