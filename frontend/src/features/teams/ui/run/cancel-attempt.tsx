import { useEffect, useId, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { useAttemptField, useTeamActions, type TeamsErrorInfo } from "@/platform/entities";
import { errorInfoOf } from "../../model/errors";
import { formatCount, formatMicrocost, formatSeconds } from "../../model/format";
import { GatedAction } from "../gated-action";
import { RuntimeNotice } from "../runtime-notice";
import { useTeamCurrency, type RunContext } from "./run-hooks";

const CARD_SELECTOR = "[data-attempt-card]";

/**
 * What a cancel can cost, in this attempt's own figures. A cancel during a model call ends with no usage report, and
 * an attempt that settles without one keeps its whole reservation against the team budget. It subscribes on its own,
 * so the form shell does not re-render with the reservation.
 */
function CancelCost({ workspace, teamId, attemptId }: RunContext & { attemptId: string }) {
  const reservation = useAttemptField(workspace, teamId, attemptId, "reservation");
  const currency = useTeamCurrency(workspace, teamId);
  const figures = reservation
    ? ` (${formatCount(reservation.tokens)} tokens, ${formatMicrocost(reservation.costMicrounits, currency)}, ${formatSeconds(reservation.elapsedSeconds)})`
    : "";
  return (
    <p className="text-sm break-words">
      If the model call has already started, the runtime may receive no usage report; the full reservation{figures} then
      counts against the team budget.
    </p>
  );
}

/**
 * Cancel one attempt. The runtime needs a reason, so the control opens a short form first. The command id is
 * created once per form and reused when the same request is sent again. Sending again after an uncertain failure
 * is safe: the domain reports success when the cancellation had already committed, instead of a conflict. A
 * conflict that is new is shown as the runtime states it; the domain reloads the team and nothing is resubmitted.
 *
 * Focus: "Keep running" returns focus to the "Cancel attempt" control that opened the form. After a send the
 * control may no longer exist (a cancelled attempt has no cancel action), so focus goes to the attempt card.
 */
export function CancelAttempt({ workspace, teamId, attemptId }: RunContext & { attemptId: string }) {
  const actions = useTeamActions();
  const [open, setOpen] = useState(false);
  const [reason, setReason] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<TeamsErrorInfo | null>(null);
  const [commandId, setCommandId] = useState(() => crypto.randomUUID());
  const fieldId = useId();
  const triggerRef = useRef<HTMLButtonElement>(null);
  const formRef = useRef<HTMLFormElement>(null);
  const restoreFocus = useRef(false);
  const trimmed = reason.trim();

  useEffect(() => {
    if (!open && restoreFocus.current) {
      restoreFocus.current = false;
      triggerRef.current?.focus();
    }
  }, [open]);

  const send = async () => {
    if (trimmed === "" || busy) return;
    const card = formRef.current?.closest<HTMLElement>(CARD_SELECTOR) ?? null;
    setBusy(true);
    setError(null);
    try {
      await actions.cancelAttempt(workspace, teamId, attemptId, { reason: trimmed, commandId });
      setOpen(false);
      setReason("");
      setCommandId(crypto.randomUUID());
      card?.focus();
    } catch (failure) {
      setError(errorInfoOf(failure));
    } finally {
      setBusy(false);
    }
  };

  const keepRunning = () => {
    restoreFocus.current = true;
    setOpen(false);
  };

  if (!open) {
    return (
      <Button ref={triggerRef} type="button" variant="destructive" className="min-h-11" onClick={() => setOpen(true)}>
        Cancel attempt
      </Button>
    );
  }

  return (
    <form
      ref={formRef}
      className="min-w-0 max-w-prose space-y-2 rounded-md bg-background/70 p-3"
      onSubmit={(event) => {
        event.preventDefault();
        void send();
      }}
    >
      <label htmlFor={fieldId} className="block text-sm font-medium">
        Why cancel this attempt?
      </label>
      <Textarea
        id={fieldId}
        autoFocus
        required
        value={reason}
        onChange={(event) => setReason(event.target.value)}
        placeholder="The runtime records this reason with the cancellation."
        readOnly={busy}
      />
      <RuntimeNotice error={error} title="The runtime did not cancel this attempt" />
      <CancelCost workspace={workspace} teamId={teamId} attemptId={attemptId} />
      <div className="flex flex-wrap items-start gap-2">
        <GatedAction
          type="submit"
          variant="destructive"
          aria-busy={busy || undefined}
          reason={busy ? "Sending the cancellation." : trimmed === "" ? "Enter a reason. Spaces alone do not count." : null}
        >
          {busy ? "Sending cancellation" : "Send cancellation"}
        </GatedAction>
        <Button type="button" variant="ghost" className="min-h-11" onClick={keepRunning}>
          Keep running
        </Button>
      </div>
    </form>
  );
}
