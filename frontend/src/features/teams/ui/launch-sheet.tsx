import { useEffect, useId, useMemo, useRef, useState } from "react";
import { Loader2, Rocket } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { NativeSelect, NativeSelectOption } from "@/components/ui/native-select";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Textarea } from "@/components/ui/textarea";
import {
  useBindingField,
  useBindingIdsForPackage,
  usePreferredBindingId,
  useTeamActions,
  useTeamDefinitionField,
  type TeamMemberSpec,
  type TeamsErrorInfo,
} from "@/platform/entities";
import { errorInfoOf } from "../model/errors";
import { lastSegment } from "../model/format";
import { parseInput, slotProblem } from "../model/launch-input";
import { useDefinitionRefs } from "../model/use-definition";
import { useActionGate } from "../model/use-readiness";
import { useCommandId } from "../model/use-sheet-helpers";
import { GatedAction } from "./gated-action";
import { RuntimeNotice } from "./runtime-notice";
import { SheetActions } from "./sheet-actions";

function BindingOption({ workspace, bindingId }: { workspace: string; bindingId: string }) {
  const revision = useBindingField(workspace, bindingId, "revision");
  const supported = useBindingField(workspace, bindingId, "activationSupported");
  return (
    <NativeSelectOption value={bindingId}>
      {lastSegment(bindingId)} rev {revision}{supported === false ? ", cannot run teams" : ""}
    </NativeSelectOption>
  );
}

function SlotEntry({ member, value, onChange }: { member: TeamMemberSpec; value: string; onChange: (value: string) => void }) {
  const id = useId();
  const problem = slotProblem(value, member);
  if (member.min === member.max) {
    // The definition allows exactly one count, so there is nothing to choose; the value is shown, not entered.
    return (
      <div className="space-y-1">
        <p className="text-sm font-medium">{member.role} members: {member.min}</p>
        <p className="text-xs text-muted-foreground">{member.kind}. Fixed at {member.min} by the team definition.</p>
      </div>
    );
  }
  return (
    <div className="space-y-1.5">
      <Label htmlFor={id}>{member.role} members</Label>
      <Input id={id} className="min-h-11" inputMode="numeric" value={value} aria-invalid={problem ? true : undefined}
        aria-describedby={`${id}-hint`} onChange={(event) => onChange(event.target.value)} />
      <p id={`${id}-hint`} className={problem ? "text-xs text-destructive" : "text-xs text-muted-foreground"}>
        {problem ?? `${member.kind}. Between ${member.min} and ${member.max}.`}
      </p>
    </div>
  );
}

interface LaunchFormProps {
  workspace: string;
  defKey: string;
  onClose: () => void;
  onLaunched: (teamId: string) => void;
}

function LaunchForm({ workspace, defKey, onClose, onLaunched }: LaunchFormProps) {
  const actions = useTeamActions();
  const commandId = useCommandId();
  const inputId = useId();
  const bindingSelectId = useId();
  const blockReasonId = useId();
  const refs = useDefinitionRefs(defKey);
  const members = useTeamDefinitionField(defKey, "members");
  const definitionId = useTeamDefinitionField(defKey, "definitionId");
  const version = useTeamDefinitionField(defKey, "version");
  const digest = useTeamDefinitionField(defKey, "digest");
  const bindingIds = useBindingIdsForPackage(workspace, refs?.packageKey ?? "");
  const preferredBindingId = usePreferredBindingId(workspace, refs?.packageKey ?? "");

  const [bindingChoice, setBindingChoice] = useState("");
  const [slots, setSlots] = useState<Record<string, string>>({});
  const [source, setSource] = useState("{}");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<TeamsErrorInfo | null>(null);
  // The form unmounts when the sheet closes. A launch that resolves after that must not navigate away from where the operator is.
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; };
  }, []);

  const bindingId = bindingIds.includes(bindingChoice) ? bindingChoice : (preferredBindingId ?? "");
  const bindingRuns = useBindingField(workspace, bindingId, "activationSupported");
  const specs = useMemo(() => members ?? [], [members]);
  const slotText = (member: TeamMemberSpec) => slots[member.role] ?? String(member.min);
  const parsed = useMemo(() => parseInput(source), [source]);
  const slotsValid = specs.every((member) => slotProblem(slotText(member), member) === null);
  const ready = bindingId !== "" && slotsValid && parsed.ok && definitionId !== undefined && version !== undefined && digest !== undefined;

  const launch = async () => {
    if (!ready || !parsed.ok || definitionId === undefined || version === undefined || digest === undefined) return;
    setBusy(true);
    setError(null);
    try {
      const team = await actions.createTeam(workspace, {
        deploymentBindingId: bindingId,
        teamDefinition: { id: definitionId, version, digest },
        input: parsed.value,
        memberSlots: specs.map((member) => ({ role: member.role, count: Number(slotText(member)) })),
        commandId: commandId.get(),
      });
      if (mounted.current) onLaunched(team.id); // The team exists either way and appears in the list.
    } catch (failure) {
      if (mounted.current) setError(errorInfoOf(failure)); // The operator's entries stay; nothing is resubmitted.
    } finally {
      setBusy(false);
    }
  };
  const touch = () => commandId.renew(); // A changed entry is a different request, so it gets a new command id.
  const blockReason = !parsed.ok
    ? "The team input is not valid JSON."
    : !slotsValid
      ? "A member count is outside its allowed range."
      : !ready
        ? "The definition is still loading."
        : null;

  return (
    <>
      <div className="min-h-0 flex-1 space-y-5 overflow-y-auto px-4">
        <div className="space-y-1.5">
          <Label htmlFor={bindingSelectId}>Binding in workspace "{workspace}"</Label>
          <NativeSelect className="w-full" id={bindingSelectId} aria-describedby={`${bindingSelectId}-full`} value={bindingId} onChange={(event) => { setBindingChoice(event.target.value); touch(); }}>
            {bindingIds.map((id) => <BindingOption key={id} workspace={workspace} bindingId={id} />)}
          </NativeSelect>
          {bindingId !== "" && <p id={`${bindingSelectId}-full`} className="font-mono text-xs break-all text-muted-foreground">{bindingId}</p>}
          {bindingRuns === false && (
            <p role="status" className="text-xs break-words text-destructive">
              The runtime reported that it cannot activate this binding, so a team launched from it will not run. Choose another binding or create one.
            </p>
          )}
        </div>
        <fieldset className="space-y-3">
          <legend className="text-sm font-semibold">Members per role</legend>
          {specs.map((member) => (
            <SlotEntry key={member.role} member={member} value={slotText(member)}
              onChange={(value) => { setSlots({ ...slots, [member.role]: value }); touch(); }} />
          ))}
        </fieldset>
        <div className="space-y-1.5">
          <Label htmlFor={inputId}>Team input (JSON)</Label>
          <Textarea id={inputId} rows={8} spellCheck={false} className="font-mono text-sm" value={source}
            aria-invalid={parsed.ok ? undefined : true} aria-describedby={`${inputId}-status`}
            onChange={(event) => { setSource(event.target.value); touch(); }} />
          <p id={`${inputId}-status`} className={parsed.ok ? "text-xs text-muted-foreground" : "text-xs break-words text-destructive"}>
            {parsed.ok ? "Valid JSON. This is the input the team starts with." : `Not valid JSON: ${parsed.message}`}
          </p>
        </div>
        <RuntimeNotice error={error} title="The runtime refused the launch" />
      </div>
      <SheetActions
        reason={blockReason}
        reasonId={blockReasonId}
        primary={
          <GatedAction reason={blockReason} externalReasonId={blockReasonId} onClick={() => { if (!busy) void launch(); }}>
            {busy ? <Loader2 aria-hidden="true" className="motion-safe:animate-spin" /> : <Rocket aria-hidden="true" />}
            Launch team
          </GatedAction>
        }
        cancelDisabled={busy}
        onCancel={onClose}
      />
    </>
  );
}

function SheetBody(props: LaunchFormProps) {
  const refs = useDefinitionRefs(props.defKey);
  const bindingCount = useBindingIdsForPackage(props.workspace, refs?.packageKey ?? "").length;
  const gate = useActionGate(props.workspace, "launch");
  const reason = gate ?? (bindingCount === 0 ? `There is no binding in workspace "${props.workspace}" for this package. Create one first.` : null);
  if (reason) {
    return (
      <div className="space-y-4 px-4">
        <p role="alert" className="rounded-lg bg-muted p-3 text-sm break-words">Launching is unavailable. {reason}</p>
        <Button type="button" variant="secondary" className="min-h-11" onClick={props.onClose}>Close</Button>
      </div>
    );
  }
  return <LaunchForm {...props} />;
}

interface LaunchSheetProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  workspace: string;
  defKey: string;
  onLaunched: (teamId: string) => void;
}

export function LaunchSheet({ open, onOpenChange, workspace, defKey, onLaunched }: LaunchSheetProps) {
  // After a launch the button that opened this sheet is gone, and the definition heading that shares the
  // detail-heading id may still be in the page while it is being replaced. So the sheet moves no focus on that
  // close; the new team's heading focuses itself when it mounts (`TeamDetail` `focusHeading`).
  const launched = useRef(false);
  const finalFocus = () => {
    if (!launched.current) return true;
    launched.current = false;
    return false;
  };
  const launchedTeam = (teamId: string) => {
    launched.current = true;
    onLaunched(teamId);
  };
  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="flex w-full flex-col sm:max-w-lg" finalFocus={finalFocus}>
        <SheetHeader>
          <SheetTitle>Launch team</SheetTitle>
          <SheetDescription>Create a team from this definition and a binding in workspace "{workspace}".</SheetDescription>
        </SheetHeader>
        {open && <SheetBody workspace={workspace} defKey={defKey} onClose={() => onOpenChange(false)} onLaunched={launchedTeam} />}
      </SheetContent>
    </Sheet>
  );
}
