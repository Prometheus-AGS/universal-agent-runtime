import { useId, useState, type MouseEvent } from "react";
import { CheckCircle2, Circle, Loader2, OctagonX } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  useRoutingOwedStep,
  useRoutingProgressField,
  useRoutingRunningStep,
  useRoutingTestTaskIds,
  useTeamActions,
} from "@/platform/entities";
import { useActionGate } from "../../model/use-readiness";
import { GatedAction } from "../gated-action";
import { RuntimeNotice } from "../runtime-notice";
import type { RunContext } from "./run-hooks";
import {
  STEP_FAILURE_LABEL,
  STEP_LABEL,
  STEP_ORDER,
  classifyStepFailure,
  stepState,
  stepsDone,
  type StepFailure,
  type StepName,
  type StepState,
} from "./run-model";

interface TestRef extends RunContext {
  taskId: string;
}

const STATE_TEXT: Record<StepState, string> = {
  done: "Done",
  running: "Sending",
  failed: "Failed",
  next: "Owed",
  later: "Not started",
};

function StepIcon({ state }: { state: StepState }) {
  const className = "size-4 shrink-0";
  if (state === "done") return <CheckCircle2 aria-hidden="true" className={`${className} text-success`} />;
  if (state === "running") return <Loader2 aria-hidden="true" className={`${className} motion-safe:animate-spin`} />;
  if (state === "failed") return <OctagonX aria-hidden="true" className={`${className} text-destructive`} />;
  return <Circle aria-hidden="true" className={`${className} text-muted-foreground`} />;
}

/**
 * Sends exactly one step, the one still owed ("Create task", "Claim" or "Admit"), with the command id already recorded
 * for it, through the domain's resume action. It never creates a second task. When that step succeeds, the next owed step
 * shows up as its own action. Renders nothing when nothing is owed.
 */
export function OwedStepButton({ workspace, teamId, taskId }: TestRef) {
  const actions = useTeamActions();
  const owed = useRoutingOwedStep(workspace, teamId, taskId);
  const running = useRoutingRunningStep(workspace, teamId, taskId);
  const step = useRoutingProgressField(workspace, teamId, taskId, "step");
  const gate = useActionGate(workspace, "routing");
  const [busy, setBusy] = useState(false);
  if (owed === null) return null;
  const sending = busy || running !== null;
  const reason = sending ? `Sending the step: ${STEP_LABEL[running ?? owed].toLowerCase()}.` : gate;
  const resume = async () => {
    setBusy(true);
    try {
      await actions.resumeRoutingTest(workspace, teamId, taskId);
      await actions.refreshTeamRunNow(workspace, teamId);
    } catch {
      // The domain records a refused step in the progress record; the panel above shows it.
    } finally {
      setBusy(false);
    }
  };
  return (
    <GatedAction reason={reason} aria-busy={sending || undefined} onClick={() => void resume()}>
      {step === "failed" ? "Retry step" : "Send step"}: {STEP_LABEL[owed].toLowerCase()}
    </GatedAction>
  );
}

/**
 * Removes only this console's record of the test (the stored progress), so a test the runtime refused at the first step,
 * or one left half done, does not stay as an owed step for good. It sends nothing to the runtime. Focus moves to the
 * heading of the surrounding section, because the control is removed with the record.
 */
function DismissTest({ workspace, teamId, taskId }: TestRef) {
  const actions = useTeamActions();
  const descriptionId = useId();
  const dismiss = (event: MouseEvent<HTMLButtonElement>) => {
    const container = event.currentTarget.closest<HTMLElement>("section, article");
    const heading = container?.querySelector<HTMLElement>("h3, h4, h5") ?? null;
    actions.dismissRoutingTest(workspace, teamId, taskId);
    if (heading) {
      heading.tabIndex = -1;
      heading.focus();
    }
  };
  return (
    <div className="flex min-w-0 flex-col items-start gap-1.5">
      <Button type="button" variant="ghost" className="min-h-11" aria-describedby={descriptionId} onClick={dismiss}>
        Dismiss this test
      </Button>
      <p id={descriptionId} className="max-w-prose text-xs text-muted-foreground">
        Dismissing removes only this console&rsquo;s record of the test. It does not undo anything the runtime already
        committed, and a task it created stays in the team.
      </p>
    </div>
  );
}

/** The action for the step still owed, and a way to drop a test that cannot or should not be finished. */
function OwedStepActions({ workspace, teamId, taskId }: TestRef) {
  const owed = useRoutingOwedStep(workspace, teamId, taskId);
  const running = useRoutingRunningStep(workspace, teamId, taskId);
  if (owed === null) return null;
  return (
    <div className="space-y-2">
      <OwedStepButton workspace={workspace} teamId={teamId} taskId={taskId} />
      <p className="max-w-prose text-xs text-muted-foreground">
        Sends only this step. Any step after it appears as its own action once this one succeeds.
      </p>
      {running === null ? <DismissTest workspace={workspace} teamId={teamId} taskId={taskId} /> : null}
    </div>
  );
}

function Step({ name, state, failure }: { name: StepName; state: StepState; failure: StepFailure | null }) {
  const text = state === "failed" && failure ? STEP_FAILURE_LABEL[failure] : STATE_TEXT[state];
  return (
    <li className="flex items-center gap-2 text-sm">
      <StepIcon state={state} />
      <span className="font-medium">{STEP_LABEL[name]}</span>
      <span className="text-xs text-muted-foreground">{text}</span>
    </li>
  );
}

function failureTitle(failure: StepFailure, failedStep: StepName | null): string {
  const step = failedStep ? `: ${STEP_LABEL[failedStep].toLowerCase()}` : "";
  if (failure === "not_sent") return `The console did not send the step${step}`;
  if (failure === "unknown_outcome") return `The outcome of the step is unknown${step}`;
  return failedStep ? `The runtime refused the step${step}` : "The runtime refused this test";
}

/** What retrying means for each way a step can fail. */
function failureNote(failure: StepFailure, failedStep: StepName | null): string {
  const task = failedStep === "create" ? "" : "The task stays in the team. ";
  if (failure === "not_sent") return `${task}Nothing reached the runtime. Retrying sends only this step.`;
  if (failure === "unknown_outcome") {
    return `${task}The runtime may have committed this step. Retrying sends only this step again, with the same command id; no second task is created.`;
  }
  return `${task}Retrying sends only the owed step with the same command id; no second task is created.`;
}

/**
 * The step reached by one routing test: create, claim, admit, each with icon and text, the runtime's refusal when
 * a step failed, and the action for the step still owed. Progress comes from the stored record, not from the task.
 */
export function RoutingProgressPanel({ workspace, teamId, taskId }: TestRef) {
  const step = useRoutingProgressField(workspace, teamId, taskId, "step");
  const failedStep = useRoutingProgressField(workspace, teamId, taskId, "failedStep");
  const lastError = useRoutingProgressField(workspace, teamId, taskId, "lastError");
  const request = useRoutingProgressField(workspace, teamId, taskId, "request");
  const role = useRoutingProgressField(workspace, teamId, taskId, "role");
  const running = useRoutingRunningStep(workspace, teamId, taskId);
  if (step === undefined) return null;
  const done = stepsDone(step, failedStep ?? null);
  const failure = lastError ? classifyStepFailure(lastError) : null;
  return (
    <div className="min-w-0 space-y-2 rounded-md bg-background/70 p-3">
      <p className="text-sm font-medium">Latest test{role ? ` for the ${role} role` : ""}</p>
      {request ? <p className="max-w-prose text-xs break-words text-muted-foreground">{request}</p> : null}
      <ol aria-label="Steps of the latest test" className="space-y-1">
        {STEP_ORDER.map((name, index) => (
          <Step key={name} name={name} state={stepState(index, done, step, running)} failure={failure} />
        ))}
      </ol>
      {step === "admitted" ? (
        <p role="status" className="text-sm">Admitted. The attempt appears in the lane of the member it was assigned to.</p>
      ) : null}
      {lastError && failure ? (
        <>
          <RuntimeNotice error={lastError} title={failureTitle(failure, failedStep ?? null)} />
          <p className="max-w-prose text-xs text-muted-foreground">{failureNote(failure, failedStep ?? null)}</p>
        </>
      ) : null}
      <OwedStepActions workspace={workspace} teamId={teamId} taskId={taskId} />
    </div>
  );
}

/**
 * A routing-test task that is created or claimed but not admitted, shown where it belongs: in the lane of the member
 * it was claimed for, or (with `memberId` null) under its role while it has no member. It renders nothing for any
 * other test.
 */
export function OwedTaskRow({ workspace, teamId, taskId, memberId }: TestRef & { memberId: string | null }) {
  const owed = useRoutingOwedStep(workspace, teamId, taskId);
  const assigned = useRoutingProgressField(workspace, teamId, taskId, "memberId");
  const title = useRoutingProgressField(workspace, teamId, taskId, "title");
  const role = useRoutingProgressField(workspace, teamId, taskId, "role");
  const lastError = useRoutingProgressField(workspace, teamId, taskId, "lastError");
  if (owed === null || (assigned ?? null) !== memberId) return null;
  return (
    <div className="min-w-0 space-y-1.5 rounded-md bg-background/70 p-3">
      <p className="text-sm break-words">
        <span className="font-medium">{title ?? "Routing test"}</span>
        <span className="text-xs text-muted-foreground">
          {memberId === null ? ` (role ${role ?? "unknown"}, no member yet)` : ""}: not admitted, {STEP_LABEL[owed].toLowerCase()} is owed
        </span>
      </p>
      {lastError ? (
        <p className="text-xs break-words text-destructive">
          {STEP_FAILURE_LABEL[classifyStepFailure(lastError)]}: {lastError.message}
        </p>
      ) : null}
      <OwedStepActions workspace={workspace} teamId={teamId} taskId={taskId} />
    </div>
  );
}

/** Tests that still owe a step, for one member's lane or (with `memberId` null) for tasks with no member. */
export function OwedTasks({ workspace, teamId, memberId }: RunContext & { memberId: string | null }) {
  const taskIds = useRoutingTestTaskIds(workspace, teamId);
  if (taskIds.length === 0) return null;
  return (
    <div className="space-y-2">
      {taskIds.map((taskId) => (
        <OwedTaskRow key={taskId} workspace={workspace} teamId={teamId} taskId={taskId} memberId={memberId} />
      ))}
    </div>
  );
}
