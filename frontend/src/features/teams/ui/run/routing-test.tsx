import { useCallback, useEffect, useId, useRef, useState, type ReactNode } from "react";
import { ChevronDown, ChevronRight, Send } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { NativeSelect, NativeSelectOption } from "@/components/ui/native-select";
import {
  DEFAULT_ROUTING_RESERVATION,
  definitionKey,
  teamLoadKeys,
  useExportedCoordinatorRole,
  useMemberIds,
  useRoutingOwedStep,
  useRoutingTestTaskIds,
  useTeamActions,
  useTeamDefinitionField,
  useTeamField,
  useTeamLoadError,
  useTeamLoadStatus,
  type TeamsErrorInfo,
} from "@/platform/entities";
import { errorInfoOf } from "../../model/errors";
import { formatMicrocost } from "../../model/format";
import { useActionGate } from "../../model/use-readiness";
import { GatedAction } from "../gated-action";
import { RuntimeNotice } from "../runtime-notice";
import { Collect } from "./collect";
import { RoutingProgressPanel } from "./routing-progress";
import { AdmittedLaneReveal } from "./lane-reveal";
import { ConsequenceLine, ReservationFields, RemainingLine } from "./routing-reservation";
import { NO_ROLES, addRole, useMemberRole, useRemaining, useTeamCurrency, type RoleHolders, type RunContext } from "./run-hooks";
import { parseReservation, reservationOverrun, reservationToDraft, type ReservationDraft } from "./run-model";

const DEFAULT_DRAFT: ReservationDraft = reservationToDraft(DEFAULT_ROUTING_RESERVATION);

interface FormProps extends RunContext {
  roles: readonly RoleHolders[];
  /** The coordinator role from the package export, or null while it is unloaded, refused or names none. */
  coordinatorRole: string | null;
  /** Why the team definition rules out any test (budget currency), or null. */
  definitionReason: string | null;
  /** Called when a test is sent, so the lane reveal knows the next admission is the operator's own. */
  onSend: () => void;
}

/** The one-line reason the submit control is unavailable, in order of what to fix first; null when it can run. */
function blockReason(parts: {
  gate: string | null;
  definitionReason: string | null;
  sending: boolean;
  role: string;
  hasRoles: boolean;
  holders: RoleHolders | undefined;
  request: string;
  reservationProblem: string | null;
  overrun: string | null;
  slots: number | null;
}): string | null {
  if (parts.gate) return parts.gate;
  if (parts.definitionReason) return parts.definitionReason;
  if (parts.sending) return "Sending the test.";
  if (parts.role === "") return parts.hasRoles ? "Choose the role to send the test to." : "This team has no members to send a test to.";
  if (!parts.holders || parts.holders.available === 0) return `No available member holds the role "${parts.role}".`;
  if (parts.slots === 0) return "No task slots are left in this team. Every test uses one and none can be given back.";
  if (parts.request.trim() === "") return "Enter the request to send.";
  return parts.reservationProblem ?? parts.overrun;
}

function RoutingForm({ workspace, teamId, roles, coordinatorRole, definitionReason, onSend }: FormProps) {
  const actions = useTeamActions();
  const gate = useActionGate(workspace, "routing");
  const remaining = useRemaining({ workspace, teamId });
  const currency = useTeamCurrency(workspace, teamId);
  const [roleChoice, setRoleChoice] = useState<string | null>(null);
  const [request, setRequest] = useState("");
  const [draft, setDraft] = useState<ReservationDraft>(DEFAULT_DRAFT);
  const [sending, setSending] = useState(false);
  const [failure, setFailure] = useState<TeamsErrorInfo | null>(null);
  const ids = useId();

  // Only a known coordinator is preselected. Without one no role is chosen for the operator.
  const defaultRole = roles.find((entry) => entry.role === coordinatorRole)?.role ?? "";
  const role = roleChoice !== null && roles.some((entry) => entry.role === roleChoice) ? roleChoice : defaultRole;
  const parsed = parseReservation(draft);
  const reason = blockReason({
    gate,
    definitionReason,
    sending,
    role,
    hasRoles: roles.length > 0,
    holders: roles.find((entry) => entry.role === role),
    request,
    reservationProblem: parsed.ok ? null : parsed.problem,
    overrun: parsed.ok ? reservationOverrun(parsed.value, remaining, (micro) => formatMicrocost(micro, currency)) : null,
    slots: remaining.slots,
  });

  const submit = async () => {
    if (!parsed.ok || reason !== null) return;
    onSend();
    setSending(true);
    setFailure(null);
    try {
      // The domain records every refusal in the test's progress record and resolves; only a failure before
      // the test could be recorded (no verified owner) throws.
      await actions.startRoutingTest({ workspace, teamId, role, request: request.trim(), reservation: parsed.value });
      setRequest("");
    } catch (error) {
      setFailure(errorInfoOf(error));
    } finally {
      setSending(false);
      // The run may now be live again; wake the watcher in case it had stopped.
      void actions.refreshTeamRunNow(workspace, teamId).catch(() => undefined);
    }
  };

  return (
    <form
      className="space-y-3"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <div className="max-w-sm space-y-1">
        <label htmlFor={`${ids}-role`} className="text-sm font-medium">Send to role</label>
        <NativeSelect id={`${ids}-role`} className="w-full" value={role} onChange={(event) => setRoleChoice(event.target.value)}>
          {role === "" ? <NativeSelectOption value="">Choose a role</NativeSelectOption> : null}
          {roles.map((entry) => (
            <NativeSelectOption key={entry.role} value={entry.role}>
              {entry.role}
              {entry.role === coordinatorRole ? " (coordinator)" : ""}
            </NativeSelectOption>
          ))}
        </NativeSelect>
        {coordinatorRole === null ? (
          <p className="max-w-prose text-xs text-muted-foreground">
            The coordinator role is not known: the package export has not loaded, was refused, or names none. No role is preselected.
          </p>
        ) : null}
      </div>
      <div className="max-w-prose space-y-1">
        <label htmlFor={`${ids}-request`} className="text-sm font-medium">Request</label>
        <Textarea
          id={`${ids}-request`}
          value={request}
          onChange={(event) => setRequest(event.target.value)}
          placeholder="What should this member do?"
        />
      </div>
      <fieldset className="min-w-0 space-y-2">
        <legend className="text-sm font-medium">Reservation held while the attempt runs</legend>
        <ReservationFields draft={draft} currency={currency} disabled={sending} onChange={setDraft} />
        <RemainingLine workspace={workspace} teamId={teamId} />
      </fieldset>
      <RuntimeNotice error={failure} title="The test could not be started" />
      <ConsequenceLine workspace={workspace} teamId={teamId} value={parsed.ok ? parsed.value : null} />
      <GatedAction
        type="submit"
        // Filled while it can run, quiet and dimmed while it cannot, so the two states never look alike.
        variant={reason === null ? "default" : "secondary"}
        className={reason === null ? "font-semibold" : undefined}
        reason={reason}
        aria-busy={sending || undefined}
      >
        <Send aria-hidden="true" />
        Reserve and send
      </GatedAction>
    </form>
  );
}

/** A failed definition or package-export load, in the runtime's words, with one retry for whatever failed. */
function SetupLoadProblems({
  definitionsError,
  exportError,
  onRetry,
}: {
  definitionsError: TeamsErrorInfo | null;
  exportError: TeamsErrorInfo | null;
  onRetry: () => void;
}) {
  if (!definitionsError && !exportError) return null;
  return (
    <div role="alert" className="max-w-prose space-y-2 rounded-md bg-destructive/10 p-3 text-sm text-destructive">
      {definitionsError ? <p className="break-words">The team definition did not load: {definitionsError.message}</p> : null}
      {exportError ? <p className="break-words">The package export did not load: {exportError.message}</p> : null}
      <Button type="button" variant="secondary" className="min-h-11" onClick={onRetry}>
        Retry loading
      </Button>
    </div>
  );
}

/** Role holders from the roster; the coordinator role and the budget currency come from the package and definition. */
function RoutingSetup({ workspace, teamId, onSend }: RunContext & { onSend: () => void }) {
  const actions = useTeamActions();
  const definition = useTeamField(workspace, teamId, "definition");
  const pkg = useTeamField(workspace, teamId, "package");
  const memberIds = useMemberIds(workspace, teamId);
  const defId = definition?.id ?? "";
  const defVersion = definition?.version ?? "";
  const pkgId = pkg?.id ?? "";
  const pkgVersion = pkg?.version ?? "";
  const pkgDigest = pkg?.digest ?? "";
  const defKey = definition ? definitionKey(definition) : "";
  const pkgKey = pkg ? definitionKey(pkg) : "";
  const definitionsLoad = useTeamLoadStatus(teamLoadKeys.definitions);
  const exportLoad = useTeamLoadStatus(teamLoadKeys.packageExport(pkgKey));
  const coordinatorRole = useExportedCoordinatorRole(pkgKey, { id: defId, version: defVersion });
  const budget = useTeamDefinitionField(defKey, "budget");
  const definitionsError = useTeamLoadError(teamLoadKeys.definitions);
  const exportError = useTeamLoadError(teamLoadKeys.packageExport(pkgKey));

  useEffect(() => {
    if (definitionsLoad === "idle") void actions.loadTeamDefinitions().catch(() => undefined); // Recorded in its load state.
  }, [actions, definitionsLoad]);
  useEffect(() => {
    if (pkgId !== "" && exportLoad === "idle") {
      void actions.loadPackageExport({ id: pkgId, version: pkgVersion, digest: pkgDigest }).catch(() => undefined);
    }
  }, [actions, exportLoad, pkgId, pkgVersion, pkgDigest]);

  const retry = () => {
    if (definitionsLoad === "error") void actions.loadTeamDefinitions().catch(() => undefined); // Recorded in its load state.
    if (pkgId !== "" && exportLoad === "error") {
      void actions.loadPackageExport({ id: pkgId, version: pkgVersion, digest: pkgDigest }).catch(() => undefined);
    }
  };

  const definitionReason =
    budget === undefined
      ? definitionsLoad === "error"
        ? "The team definition did not load, so its budget currency is unknown."
        : definitionsLoad === "ready" && defKey !== ""
          ? "The team definition was not found among the installed definitions, so its budget currency is unknown."
          : "Waiting for the team definition to load."
      : budget.currency === "USD"
        ? null
        : `The team definition's budget currency is ${budget.currency ?? "not set"}. The runtime admits work only against USD budgets, and the console cannot change a definition.`;

  return (
    <div className="space-y-3">
      <SetupLoadProblems definitionsError={definitionsError} exportError={exportError} onRetry={retry} />
      <Collect ids={memberIds} context={{ workspace, teamId }} useValue={useMemberRole} reduce={addRole} initial={NO_ROLES}>
        {(roles) => (
          <RoutingForm
            workspace={workspace}
            teamId={teamId}
            roles={roles}
            coordinatorRole={coordinatorRole}
            definitionReason={definitionReason}
            onSend={onSend}
          />
        )}
      </Collect>
    </div>
  );
}

function LatestTest({ workspace, teamId }: RunContext) {
  const taskIds = useRoutingTestTaskIds(workspace, teamId);
  if (taskIds.length === 0) return null;
  return <RoutingProgressPanel workspace={workspace} teamId={teamId} taskId={taskIds[taskIds.length - 1]} />;
}

interface RoutingDisclosureProps extends RunContext {
  panelId: string;
  /** The operator asked for the form. */
  requested: boolean;
  onToggle: () => void;
  children: ReactNode;
}

/**
 * The "Send a test request" button and the form it opens. The form stays mounted while closed, so a draft and a
 * failure notice survive a collapse, and it cannot be collapsed while the latest test still owes a step (running,
 * refused or half done), because the retry and the runtime's refusal live with it. This is the one component that
 * follows the latest test's owed step; the form itself is passed in and does not re-render with it.
 */
function RoutingDisclosure({ workspace, teamId, panelId, requested, onToggle, children }: RoutingDisclosureProps) {
  const taskIds = useRoutingTestTaskIds(workspace, teamId);
  const owed = useRoutingOwedStep(workspace, teamId, taskIds.length > 0 ? taskIds[taskIds.length - 1] : "");
  const pinned = owed !== null;
  const open = requested || pinned;
  const Chevron = open ? ChevronDown : ChevronRight;
  const noteId = useId();
  return (
    <div className="space-y-3">
      <div className="flex min-w-0 flex-col items-start gap-1.5">
        <Button
          type="button"
          variant={open ? "secondary" : "default"}
          className="min-h-11"
          aria-expanded={open}
          aria-controls={panelId}
          aria-disabled={pinned || undefined}
          aria-describedby={pinned ? noteId : undefined}
          onClick={() => {
            if (!pinned) onToggle();
          }}
        >
          <Chevron aria-hidden="true" />
          Send a test request
        </Button>
        {pinned ? (
          <p id={noteId} className="max-w-prose text-xs text-muted-foreground">
            Stays open while the latest test still owes a step.
          </p>
        ) : null}
      </div>
      <div id={panelId} hidden={!open}>
        {children}
      </div>
    </div>
  );
}

/**
 * Send a test request to a member role. The member lanes come first on the page; this is a collapsed control below
 * them. Submitting runs three explicit commands (create the task, claim it for a member holding the role, admit it
 * with the reservation); the step reached is shown, and a refused step stays owed and is retried alone. The
 * reservation is real budget: the form states its cost before it is sent. When the test is admitted the member's lane
 * is brought into view and the form folds away.
 */
export function RoutingTestConsole({ workspace, teamId }: RunContext) {
  const headingId = useId();
  const panelId = useId();
  const [requested, setRequested] = useState(false);
  // Set when this console sent a test (or saw one still owing a step); read once by the reveal when it is admitted.
  const armedRef = useRef(false);
  const onToggle = useCallback(() => setRequested((current) => !current), []);
  const onSend = useCallback(() => {
    armedRef.current = true;
  }, []);
  const onRevealed = useCallback(() => setRequested(false), []);
  return (
    <section aria-labelledby={headingId} className="space-y-3 rounded-lg bg-muted/40 p-3 sm:p-4">
      <div className="space-y-1">
        <h4 id={headingId} className="text-sm font-semibold">Routing test</h4>
        <p className="max-w-prose text-xs text-muted-foreground">
          Creates a task for a role, claims it for a member, and admits it. The reservation counts against the team
          budget, and each test uses one task slot that cannot be given back.
        </p>
      </div>
      <RoutingDisclosure workspace={workspace} teamId={teamId} panelId={panelId} requested={requested} onToggle={onToggle}>
        <RoutingSetup workspace={workspace} teamId={teamId} onSend={onSend} />
      </RoutingDisclosure>
      <LatestTest workspace={workspace} teamId={teamId} />
      <AdmittedLaneReveal workspace={workspace} teamId={teamId} armedRef={armedRef} onRevealed={onRevealed} />
    </section>
  );
}
