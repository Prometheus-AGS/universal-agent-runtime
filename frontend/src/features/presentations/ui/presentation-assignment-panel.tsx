import { memo, useEffect, useId, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  usePresentationActions, usePresentationCatalogField, usePresentationField, usePresentationIds,
  usePresentationAssignmentActions, usePresentationAssignmentReady, usePresentationAssignmentLoadError,
  usePresentationAssignmentField, usePresentationAssignmentMode, usePresentationAssignmentIds,
  usePresentationAssignmentMarked, usePresentationAssignmentRetainedCount, usePresentationAssignmentMatchCount,
  type PresentationAssignmentTarget, type PresentationSelectionMode,
} from "@/platform/entities";
import { PresentationConfirmation } from "./presentation-confirmation";

interface TargetProps { target: PresentationAssignmentTarget }
interface ControlProps extends TargetProps { disabled: boolean }

const MODE_HELP: Record<PresentationSelectionMode, string> = {
  inherit: "Keep parent intent without copying its IDs. Exclusions below still apply.",
  auto: "Let UAR choose a subset allowed by parent policy.",
  all: "Request every template allowed by parent policy.",
  selected: "Request only checked templates, subject to parent restrictions.",
  none: "Make no templates available at this scope.",
};

function AssignmentMode({ target, disabled }: ControlProps) {
  const id = useId();
  const mode = usePresentationAssignmentMode(target);
  const excluded = usePresentationAssignmentIds(target, true);
  const retained = usePresentationAssignmentRetainedCount(target);
  const actions = usePresentationAssignmentActions();
  return <div className="space-y-2">
    <Label htmlFor={id}>Assignment mode</Label>
    <Select value={mode ?? "inherit"} disabled={disabled}
      items={{ inherit: excluded.length ? "Inherit, with exclusions" : "Inherit", auto: "Automatic", all: "All allowed", selected: "Selected templates", none: "None" }}
      onValueChange={(value) => { if (value && value in MODE_HELP) actions.setMode(target, value as PresentationSelectionMode); }}>
      <SelectTrigger id={id} className="min-h-11 w-full" aria-describedby={`${id}-help`}><SelectValue /></SelectTrigger>
      <SelectContent>
        <SelectItem value="inherit">{excluded.length ? "Inherit, with exclusions" : "Inherit"}</SelectItem>
        <SelectItem value="auto">Automatic</SelectItem>
        <SelectItem value="all">All allowed</SelectItem>
        <SelectItem value="selected">Selected templates</SelectItem>
        <SelectItem value="none">None</SelectItem>
      </SelectContent>
    </Select>
    <p id={`${id}-help`} className="text-sm text-muted-foreground">{mode === "inherit" && target.scope === "global"
      ? "Leave selection to the runtime's automatic behavior. Exclusions below still apply." : mode ? MODE_HELP[mode] : "Assignment unavailable."}</p>
    {retained > 0 && <p className="text-xs text-muted-foreground">{retained} remembered for returning to Selected templates; these IDs are inactive in this mode.</p>}
    {target.scope === "global" && <p className="rounded-md bg-muted/60 p-3 text-sm">
      This catalog belongs to the signed-in owner; this ceiling affects the entire instance. Selected templates excludes every ID outside the selection, including other owners’ IDs. None disables templates for everyone.
    </p>}
  </div>;
}

const AssignmentRow = memo(function AssignmentRow({ target, disabled, id, search }: ControlProps & { id: string; search: string }) {
  const inputId = useId();
  const title = usePresentationField(id, "title");
  const enabled = usePresentationField(id, "enabled");
  const selected = usePresentationAssignmentMarked(target, id);
  const excluded = usePresentationAssignmentMarked(target, id, true);
  const mode = usePresentationAssignmentMode(target);
  const actions = usePresentationAssignmentActions();
  if (!`${title ?? id} ${id}`.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase())) return null;
  const unavailable = enabled !== true;
  const status = title === undefined ? "Unavailable — not in this catalog" : !enabled ? "Unavailable — disabled" : "Availability resolved at run admission";
  return <li className="flex flex-wrap items-center gap-x-2 gap-y-1 rounded-md bg-muted/40 px-3 py-2">
    <Label htmlFor={inputId} className="flex min-h-11 min-w-0 flex-1 cursor-pointer items-center gap-3 py-1">
      <Checkbox id={inputId} checked={selected} aria-describedby={`${inputId}-status`}
        disabled={disabled || mode !== "selected" || (unavailable && !selected)}
        onCheckedChange={() => actions.toggle(target, id)} />
      <span className="min-w-0 break-words text-sm">{title ?? id}</span>
    </Label>
    <Button type="button" variant="secondary" className="min-h-11 shrink-0 text-xs"
      aria-label={`${excluded ? "Remove exclusion for" : "Exclude"} ${title ?? id}`}
      disabled={disabled || (unavailable && !excluded)} onClick={() => actions.toggle(target, id, true)}>
      {excluded ? "Remove exclusion" : "Exclude"}
    </Button>
    <p id={`${inputId}-status`} className="w-full break-words text-xs text-muted-foreground">{status}{excluded ? "; excluded here in every mode" : ""}</p>
  </li>;
});

const AssignmentExclusion = memo(function AssignmentExclusion({ target, disabled, id }: ControlProps & { id: string }) {
  const title = usePresentationField(id, "title");
  const actions = usePresentationAssignmentActions();
  return <li className="flex min-w-0 items-center gap-2 text-xs"><span className="min-w-0 flex-1 break-words">Excluded: {title ?? id}</span>
    <Button type="button" variant="secondary" className="min-h-11 shrink-0 text-xs" disabled={disabled}
      aria-label={`Remove exclusion for ${title ?? id}`} onClick={() => actions.toggle(target, id, true)}>Remove</Button></li>;
});

function AssignmentChoices({ target, disabled }: ControlProps) {
  const searchId = useId();
  const [search, setSearch] = useState("");
  const catalogIds = usePresentationIds();
  const selected = usePresentationAssignmentIds(target);
  const excluded = usePresentationAssignmentIds(target, true);
  const mode = usePresentationAssignmentMode(target);
  const matches = usePresentationAssignmentMatchCount(target, search);
  const actions = usePresentationAssignmentActions();
  const ids = [...new Set([...catalogIds, ...selected, ...excluded])];
  return <div className="space-y-3">
    <p className="text-sm text-muted-foreground" aria-live="polite">
      {mode === "selected" ? `${selected.length} requested; ` : ""}{excluded.length} exclusions apply in every mode.
      {mode === "selected" && selected.length === 0 ? " No templates will be eligible at this scope." : ""}
    </p>
    {excluded.length > 0 && <ul aria-label="Active exclusions" className="space-y-1">
      {excluded.map((id) => <AssignmentExclusion key={id} target={target} disabled={disabled} id={id} />)}
    </ul>}
    <details open={mode === "selected"}>
      <summary className="min-h-11 cursor-pointer py-3 text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-ring">Browse templates and exclusions</summary>
      <div className="space-y-3 pt-2">
        <Label htmlFor={searchId}>Find a template</Label>
        <Input id={searchId} value={search} onChange={(event) => setSearch(event.target.value)} className="min-h-11" placeholder="Search title or ID" />
        {ids.length === 0 ? <p className="text-sm text-muted-foreground">No templates in this catalog. Create one to select it here.</p>
          : matches === 0 ? <div role="status"><p className="text-sm">No matching templates.</p><Button type="button" variant="secondary" className="mt-2 min-h-11" onClick={() => setSearch("")}>Clear search</Button></div>
            : <ul aria-label="Template choices" className="max-h-80 space-y-2 overflow-y-auto">{ids.map((id) => <AssignmentRow key={id} target={target} disabled={disabled} id={id} search={search} />)}</ul>}
        <a href="/admin/presentations" target="_blank" rel="noopener noreferrer" className="inline-flex min-h-11 items-center text-sm text-primary underline underline-offset-4 focus-visible:outline focus-visible:outline-2 focus-visible:outline-ring">Manage templates (opens in a new tab)</a>
      </div>
    </details>
    <Button type="button" variant="secondary" className="min-h-11" disabled={disabled} onClick={() => actions.reset(target)}>Reset assignment</Button>
    <p className="text-xs text-muted-foreground">Reset restores inheritance and clears selections and exclusions. Save assignment applies these changes.</p>
  </div>;
}

function AssignmentFields({ target }: TargetProps) {
  const status = usePresentationAssignmentField(target, "status");
  const uncertain = usePresentationAssignmentField(target, "uncertain");
  const conflict = usePresentationAssignmentField(target, "conflict");
  const recovered = usePresentationAssignmentField(target, "recovered");
  const disabled = status === "saving" || uncertain === true || conflict === true || recovered === true;
  return <><AssignmentMode target={target} disabled={disabled} /><AssignmentChoices target={target} disabled={disabled} /></>;
}

function AssignmentFooter({ target, focusUnavailable }: TargetProps & { focusUnavailable: () => void }) {
  const [confirmation, setConfirmation] = useState<"discard" | "global" | null>(null);
  const errorRef = useRef<HTMLParagraphElement>(null);
  const status = usePresentationAssignmentField(target, "status");
  const error = usePresentationAssignmentField(target, "error");
  const dirty = usePresentationAssignmentField(target, "dirty");
  const uncertain = usePresentationAssignmentField(target, "uncertain");
  const conflict = usePresentationAssignmentField(target, "conflict");
  const recovered = usePresentationAssignmentField(target, "recovered");
  const mode = usePresentationAssignmentMode(target);
  const actions = usePresentationAssignmentActions();
  const catalogActions = usePresentationActions();
  const saving = status === "saving";
  const save = async () => {
    setConfirmation(null);
    if (!await actions.save(target)) requestAnimationFrame(() => {
      if (errorRef.current) errorRef.current.focus();
      else focusUnavailable();
    });
  };
  return <div className="space-y-3 rounded-md bg-muted/30 p-3">
    <p role="status" className="text-sm text-muted-foreground">{saving ? "Waiting for the server…" : dirty ? "Unsaved assignment — retained when you leave this panel." : "Saved intent loaded. A future run resolves availability; this is not a render receipt."}</p>
    {error && <p ref={errorRef} tabIndex={-1} role="alert" className="rounded-md bg-destructive/10 p-3 text-sm text-destructive focus-visible:outline focus-visible:outline-2 focus-visible:outline-ring">{error}</p>}
    {recovered && <p className="text-sm">A draft from a previous visit was recovered after owner verification. Resume it or discard and reload saved intent.</p>}
    <div className="flex flex-wrap gap-2">
      {recovered && <Button type="button" variant="secondary" className="min-h-11" disabled={saving} onClick={() => actions.resume(target)}>Resume draft</Button>}
      {uncertain && <Button type="button" className="min-h-11" disabled={saving} onClick={() => { void actions.checkSaved(target); }}>Check saved assignment</Button>}
      <Button type="button" className="min-h-11" disabled={!dirty || saving || uncertain || conflict || recovered}
        onClick={() => { if (target.scope === "global" && (mode === "selected" || mode === "none")) setConfirmation("global"); else void save(); }}>Save assignment</Button>
      <Button type="button" variant="secondary" className="min-h-11" disabled={saving}
        onClick={() => { if (dirty) setConfirmation("discard"); else void actions.reload(target); }}>{dirty ? "Discard and reload…" : "Reload assignment"}</Button>
      <Button type="button" variant="secondary" className="min-h-11" disabled={saving} onClick={() => { void catalogActions.reload(); }}>Reload catalog</Button>
    </div>
    <PresentationConfirmation open={confirmation !== null} onOpenChange={(open) => { if (!open) setConfirmation(null); }} pending={saving}
      title={confirmation === "discard" ? "Discard this assignment draft?" : "Restrict presentations for the entire instance?"}
      description={confirmation === "discard" ? "Your unsaved selections and exclusions will be replaced with saved intent. This does not undo any write the server already received."
        : mode === "none" ? "None disables templates for everyone on this instance. This affects future runs, not only your own."
          : "Only these selected IDs may be eligible. Every ID outside the selection is excluded across the instance, including other owners’ IDs."}
      confirmLabel={confirmation === "discard" ? "Discard and reload" : "Save instance restriction"}
      onConfirm={() => { if (confirmation === "discard") { setConfirmation(null); void actions.discard(target); } else void save(); }} />
  </div>;
}

export function PresentationAssignmentPanel({ target }: TargetProps) {
  const headingId = useId();
  const unavailableRef = useRef<HTMLParagraphElement>(null);
  const ready = usePresentationAssignmentReady(target);
  const loadError = usePresentationAssignmentLoadError(target);
  const owner = usePresentationCatalogField("owner_id");
  const catalogStatus = usePresentationCatalogField("status");
  const catalogError = usePresentationCatalogField("error");
  const actions = usePresentationAssignmentActions();
  const catalogActions = usePresentationActions();
  useEffect(() => { void actions.ensureLoaded(target); }, [actions, target, owner, catalogStatus]);
  return <section aria-labelledby={headingId} className="space-y-4 rounded-lg bg-card p-4 sm:p-5">
    <div className="space-y-1">
      <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">{target.scope === "global" ? "Instance-wide ceiling · admin access required" : "Agent assignment"}</p>
      <h3 id={headingId} className="text-base font-semibold">Presentations</h3>
      <p className="text-sm text-muted-foreground">Reusable UI templates for future runs. Parent policy and client support can narrow this request; saving does not render a template.</p>
    </div>
    {ready ? <><AssignmentFields target={target} /><AssignmentFooter target={target} focusUnavailable={() => unavailableRef.current?.focus()} /></> : <div className="space-y-3" role="status">
      <p ref={unavailableRef} tabIndex={-1} className="text-sm text-muted-foreground focus-visible:outline focus-visible:outline-2 focus-visible:outline-ring">{catalogError ?? loadError ?? "Verifying the owner and loading saved intent. Retained drafts stay hidden until verification succeeds."}</p>
      <Button type="button" variant="secondary" className="min-h-11" disabled={catalogStatus === "loading"}
        onClick={() => { if (catalogStatus === "ready") void actions.reload(target); else void catalogActions.reload(); }}>{catalogStatus === "ready" ? "Reload assignment" : "Reload catalog"}</Button>
    </div>}
  </section>;
}

export function AgentPresentationAssignment({ agentId }: { agentId: string }) {
  const target = useMemo<PresentationAssignmentTarget>(() => ({ scope: "agent", agentId }), [agentId]);
  return <PresentationAssignmentPanel key={agentId} target={target} />;
}

const GLOBAL_TARGET: PresentationAssignmentTarget = { scope: "global" };
export function GlobalPresentationAssignment() {
  return <PresentationAssignmentPanel target={GLOBAL_TARGET} />;
}
