import { useId, useRef, useState } from "react";
import { ArrowLeft, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import {
  BINDING_CAPABILITY,
  useExportedModelRequirements,
  useTeamActions,
  useTeamDefinitionField,
  useTeamsHasCapability,
  useTeamsOwnerId,
  useTeamsRuntimeField,
  type BindingPreflightResponse,
  type BuiltBinding,
  type ConversionDiagnostic,
  type TeamsErrorInfo,
} from "@/platform/entities";
import {
  EXAMPLE_CONNECTION_REF,
  EXAMPLE_CREDENTIAL_REF,
  EXAMPLE_POLICY_REVISION,
  effectiveBudgetOf,
  effectiveLimitsOf,
  initialDraft,
  missingItems,
  modelBindingsOf,
  numberProblems,
  type BindingDraft,
  type ModelChoice,
  type NumberFields,
} from "../model/binding-draft";
import { bucketDiagnostics, hasBlocking } from "../model/binding-diagnostics";
import { errorInfoOf } from "../model/errors";
import { useBindGate, useDefinitionRefs } from "../model/use-definition";
import { useCommandId } from "../model/use-sheet-helpers";
import { LimitsFields, ReferenceField } from "./binding-reference-fields";
import { ModelStep } from "./binding-model-step";
import { GatedAction } from "./gated-action";
import { DiagnosticList, RuntimeNotice } from "./runtime-notice";
import { SheetActions } from "./sheet-actions";

const NO_DEFINITION = { id: "", version: "" } as const;

interface Prepared {
  built: BuiltBinding;
  preflight: BindingPreflightResponse;
}

interface BindingFormProps {
  workspace: string;
  defKey: string;
  onClose: () => void;
  onInstalled: (bindingId: string) => void;
}

function BindingForm({ workspace, defKey, onClose, onInstalled }: BindingFormProps) {
  const actions = useTeamActions();
  const commandId = useCommandId();
  const footerReasonId = useId();
  const refs = useDefinitionRefs(defKey);
  const definitionLimits = useTeamDefinitionField(defKey, "limits");
  const definitionBudget = useTeamDefinitionField(defKey, "budget");
  const requirements = useExportedModelRequirements(refs?.packageKey ?? "", refs?.definition ?? NO_DEFINITION);
  const ownerId = useTeamsOwnerId();
  const instanceId = useTeamsRuntimeField("instanceId");
  const storageBackend = useTeamsRuntimeField("catalogBackend");
  const reportsCapability = useTeamsHasCapability(BINDING_CAPABILITY);

  const [draft, setDraft] = useState<BindingDraft>(() => initialDraft(definitionLimits, definitionBudget));
  const [stage, setStage] = useState<"edit" | "review">("edit");
  const [prepared, setPrepared] = useState<Prepared | null>(null);
  // Kept across edits so Back still shows each result beside its field; replaced by the next check.
  const [lastDiagnostics, setLastDiagnostics] = useState<readonly ConversionDiagnostic[]>([]);
  const [busy, setBusy] = useState<"check" | "install" | null>(null);
  const [error, setError] = useState<TeamsErrorInfo | null>(null);
  // Step 2 holds the required references, and none is filled in at the start, so it starts open. A later
  // collapse is the operator's choice; the reason beside Check binding names what is still missing.
  const [detailsOpen, setDetailsOpen] = useState(true);
  // Counts draft edits, so a check that was in flight when the draft changed can be recognised and dropped.
  const draftVersion = useRef(0);

  const edit = (change: (current: BindingDraft) => BindingDraft) => {
    draftVersion.current += 1;
    setDraft(change);
    setPrepared((current) => (current ? null : current)); // A changed entry invalidates the checked document.
    commandId.renew();
  };
  const setChoice = (key: string, patch: Partial<ModelChoice>) =>
    edit((current) => {
      const previous: ModelChoice | undefined = current.choices[key];
      return { ...current, choices: { ...current.choices, [key]: { ...(previous ?? { alias: null, route: null }), ...patch } } };
    });
  const setNumber = (key: keyof NumberFields, value: string) =>
    edit((current) => ({ ...current, numbers: { ...current.numbers, ...({ [key]: value } as Partial<NumberFields>) } }));
  const setReference = (key: "credentialRef" | "connectionRef" | "policyRevision", value: string) =>
    edit((current) => ({ ...current, ...({ [key]: value } as Partial<BindingDraft>) }));

  const problems = numberProblems(draft.numbers);
  const missing = missingItems(draft, requirements);
  const checkReason = missing.length > 0
    ? `Still needed: ${missing.join(", ")}.${detailsOpen ? "" : " The references and limits are under Step 2."}`
    : null;
  const diagnostics = bucketDiagnostics(lastDiagnostics);

  const check = async () => {
    if (!refs || checkReason || busy !== null) return;
    const mine = draftVersion.current;
    setBusy("check");
    setError(null);
    try {
      const built = await actions.buildDeploymentBinding({
        package: refs.package,
        ownerId: ownerId ?? "",
        workspaceId: workspace,
        runtimeInstanceId: instanceId ?? "",
        runtimeCapabilities: reportsCapability ? [BINDING_CAPABILITY] : [],
        modelBindings: modelBindingsOf(draft, requirements),
        effectiveLimits: effectiveLimitsOf(draft.numbers),
        effectiveBudget: effectiveBudgetOf(draft.numbers),
        storage: { backend: storageBackend ?? "", connectionRef: draft.connectionRef.trim() },
        policyRevision: draft.policyRevision.trim(),
      });
      const preflight = await actions.preflightTeamBinding(workspace, { binding: built.document, expectedRevision: built.expectedRevision });
      if (mine !== draftVersion.current) return; // The draft changed while the runtime was checking; this result is for the older draft.
      setPrepared({ built, preflight });
      setLastDiagnostics(preflight.diagnostics);
      setStage("review");
    } catch (failure) {
      if (mine === draftVersion.current) setError(errorInfoOf(failure));
    } finally {
      setBusy(null);
    }
  };

  const install = async () => {
    if (!prepared) return;
    setBusy("install");
    setError(null);
    try {
      const response = await actions.installTeamBinding(workspace, {
        binding: prepared.built.document,
        expectedRevision: prepared.built.expectedRevision,
        commandId: commandId.get(),
      });
      onInstalled(response.binding.id);
    } catch (failure) {
      setError(errorInfoOf(failure));
    } finally {
      setBusy(null);
    }
  };

  const installable = prepared !== null && prepared.preflight.activationSupported && !hasBlocking(prepared.preflight.diagnostics);

  return (
    <>
      <div className="min-h-0 flex-1 space-y-6 overflow-y-auto px-4">
        {stage === "edit" ? (
          <>
            {diagnostics.general.length > 0 && <DiagnosticList label="General check results" diagnostics={diagnostics.general} />}
            <ModelStep requirements={requirements} choices={draft.choices} byRequirement={diagnostics.byRequirement}
              modelDiagnostics={diagnostics.models} onChange={setChoice} />
            <section aria-labelledby="binding-details-heading">
              <details open={detailsOpen} onToggle={(event) => setDetailsOpen(event.currentTarget.open)} className="rounded-lg bg-muted/40 p-3">
                <summary id="binding-details-heading" className="min-h-11 cursor-pointer content-center text-sm font-semibold focus-visible:outline-2 focus-visible:outline-ring">
                  Step 2. References (required) and limits
                </summary>
                <div className="space-y-6 pt-3">
                  <dl className="grid gap-2 text-sm sm:grid-cols-2">
                    <div><dt className="text-xs text-muted-foreground">Runtime instance id</dt><dd className="font-mono break-all">{instanceId}</dd></div>
                    <div><dt className="text-xs text-muted-foreground">Storage backend</dt><dd className="font-mono break-all">{storageBackend}</dd></div>
                  </dl>
                  <LimitsFields numbers={draft.numbers} problems={problems} diagnostics={diagnostics.limits} onChange={setNumber} />
                  <ReferenceField label="Credential reference" example={EXAMPLE_CREDENTIAL_REF} value={draft.credentialRef}
                    explanation="Points at the protected credential the runtime resolves for every model above. The secret itself is never entered here."
                    diagnostics={diagnostics.credential} onChange={(value) => setReference("credentialRef", value)} />
                  <ReferenceField label="Storage connection reference" example={EXAMPLE_CONNECTION_REF} value={draft.connectionRef}
                    explanation="Points at the protected connection the runtime uses for this binding's durable team storage."
                    diagnostics={diagnostics.connection} onChange={(value) => setReference("connectionRef", value)} />
                  <ReferenceField label="Policy revision" example={EXAMPLE_POLICY_REVISION} value={draft.policyRevision}
                    explanation="Names the policy revision the runtime applies to teams launched from this binding."
                    diagnostics={diagnostics.policy} onChange={(value) => setReference("policyRevision", value)} />
                  <p className="text-xs text-muted-foreground">
                    The runtime only checks that a reference contains "://". A wrong reference is found when work is admitted, not here.
                  </p>
                </div>
              </details>
            </section>
          </>
        ) : (
          prepared && <Review prepared={prepared} draft={draft} requirementCount={requirements.length} installable={installable} />
        )}
        <RuntimeNotice error={error} />
      </div>
      {stage === "edit" ? (
        <SheetActions
          reason={checkReason}
          reasonId={footerReasonId}
          primary={
            <GatedAction reason={checkReason} externalReasonId={footerReasonId} onClick={() => void check()}>
              {busy === "check" && <Loader2 aria-hidden="true" className="motion-safe:animate-spin" />}
              Check binding
            </GatedAction>
          }
          cancelDisabled={busy === "install"}
          onCancel={onClose}
        />
      ) : (
        <SheetActions
          reason={null} // The check result above already says why Install binding is unavailable.
          reasonId={footerReasonId}
          primary={
            <Button type="button" className="min-h-11" disabled={!installable || busy !== null} onClick={() => void install()}>
              {busy === "install" && <Loader2 aria-hidden="true" className="motion-safe:animate-spin" />}
              Install binding
            </Button>
          }
          secondary={
            <Button type="button" variant="secondary" className="min-h-11" disabled={busy !== null} onClick={() => { setStage("edit"); setError(null); }}>
              <ArrowLeft aria-hidden="true" />
              Back
            </Button>
          }
          cancelDisabled={busy === "install"}
          onCancel={onClose}
        />
      )}
    </>
  );
}

function Review({ prepared, draft, requirementCount, installable }: {
  prepared: Prepared;
  draft: BindingDraft;
  requirementCount: number;
  installable: boolean;
}) {
  const doc = prepared.built.document;
  const buckets = bucketDiagnostics(prepared.preflight.diagnostics);
  return (
    <section aria-labelledby="binding-review-heading" className="space-y-4">
      <h3 id="binding-review-heading" className="text-sm font-semibold">Binding check</h3>
      <p role="status" className="text-sm">
        {installable ? "The runtime can activate this binding." : "The runtime cannot activate this binding. Go back, change the entries and check again."}
      </p>
      <dl className="grid gap-2 text-sm">
        <div><dt className="text-xs text-muted-foreground">Binding id</dt><dd className="font-mono break-all">{doc.id}</dd></div>
        <div><dt className="text-xs text-muted-foreground">Revision</dt><dd>{doc.revision}</dd></div>
        <div>
          <dt className="text-xs text-muted-foreground">Models ({requirementCount})</dt>
          <dd>
            <ul>
              {doc.modelBindings.map((model, index) => (
                <li key={`${model.requestedAlias}:${model.providerId}/${model.modelId}:${index}`} className="font-mono text-xs break-all">{model.requestedAlias} → {model.providerId}/{model.modelId}</li>
              ))}
            </ul>
          </dd>
        </div>
        <div><dt className="text-xs text-muted-foreground">Credential reference</dt><dd className="font-mono break-all">{draft.credentialRef}</dd></div>
        <div><dt className="text-xs text-muted-foreground">Storage connection reference</dt><dd className="font-mono break-all">{draft.connectionRef}</dd></div>
        <div><dt className="text-xs text-muted-foreground">Policy revision</dt><dd className="font-mono break-all">{draft.policyRevision}</dd></div>
      </dl>
      {buckets.models.length > 0 && <DiagnosticList label="Package model checks" diagnostics={buckets.models} />}
      <DiagnosticList label="Binding check diagnostics" diagnostics={prepared.preflight.diagnostics.filter((entry) => !buckets.models.includes(entry))} />
    </section>
  );
}

interface BindingSheetProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  workspace: string;
  defKey: string;
  onInstalled: (bindingId: string) => void;
}

function Unavailable({ reason, onClose }: { reason: string; onClose: () => void }) {
  return (
    <div className="space-y-4 px-4">
      <p role="alert" className="rounded-lg bg-muted p-3 text-sm break-words">Binding creation is unavailable. {reason}</p>
      <Button type="button" variant="secondary" className="min-h-11" onClick={onClose}>Close</Button>
    </div>
  );
}

function SheetBody({ workspace, defKey, onClose, onInstalled }: BindingFormProps) {
  const reason = useBindGate(workspace, defKey);
  if (reason) return <Unavailable reason={reason} onClose={onClose} />;
  return <BindingForm workspace={workspace} defKey={defKey} onClose={onClose} onInstalled={onInstalled} />;
}

export function BindingSheet({ open, onOpenChange, workspace, defKey, onInstalled }: BindingSheetProps) {
  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="flex w-full flex-col sm:max-w-xl">
        <SheetHeader>
          <SheetTitle>Create binding</SheetTitle>
          <SheetDescription>Bind this team's model aliases to a provider and model in workspace "{workspace}".</SheetDescription>
        </SheetHeader>
        {open && <SheetBody workspace={workspace} defKey={defKey} onClose={() => onOpenChange(false)} onInstalled={onInstalled} />}
      </SheetContent>
    </Sheet>
  );
}
