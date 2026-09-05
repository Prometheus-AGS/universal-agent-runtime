import { memo, useEffect, useId, useState } from "react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  useAgentSessionDraftActions, useAgentSessionDraftStatus,
  usePresentationActions, usePresentationCatalogField, usePresentationField, usePresentationIds,
  useSessionPresentationIds, useSessionPresentationMarked, useSessionPresentationMatchCount,
  useSessionPresentationMode, useSessionPresentationReady, useSessionPresentationRetainedCount,
  useSessionPresentationError,
  type PresentationSelectionMode,
} from "@/platform/entities";

const MODE_HELP: Record<PresentationSelectionMode, string> = {
  inherit: "Keep the parent policy. Exclusions below still apply.",
  auto: "Let UAR choose a subset of the templates allowed by parent policy.",
  all: "Request every template allowed by parent policy.",
  selected: "Request only the checked templates, subject to parent restrictions.",
  none: "Make no templates available for this session.",
};

interface SessionControlProps { draftId: string; disabled: boolean }

function SessionPresentationMode({ draftId, disabled }: SessionControlProps) {
  const id = useId();
  const mode = useSessionPresentationMode(draftId);
  const exclusions = useSessionPresentationIds(draftId, true);
  const retained = useSessionPresentationRetainedCount(draftId);
  const actions = useAgentSessionDraftActions();
  return <div className="space-y-2">
    <Label htmlFor={id}>Assignment mode</Label>
    <Select value={mode ?? "inherit"} disabled={disabled || mode === undefined}
      items={{ inherit: exclusions.length ? "Inherit, with exclusions" : "Inherit", auto: "Automatic", all: "All allowed", selected: "Selected templates", none: "None" }}
      onValueChange={(value) => { if (value && value in MODE_HELP) actions.setPresentationMode(draftId, value as PresentationSelectionMode); }}>
      <SelectTrigger id={id} aria-describedby={`${id}-help`} className="min-h-11 w-full"><SelectValue /></SelectTrigger>
      <SelectContent>
        <SelectItem value="inherit">{exclusions.length ? "Inherit, with exclusions" : "Inherit"}</SelectItem>
        <SelectItem value="auto">Automatic</SelectItem>
        <SelectItem value="all">All allowed</SelectItem>
        <SelectItem value="selected">Selected templates</SelectItem>
        <SelectItem value="none">None</SelectItem>
      </SelectContent>
    </Select>
    <p id={`${id}-help`} className="text-xs text-muted-foreground">{mode ? MODE_HELP[mode] : "Assignment is not yet available."}</p>
    {retained > 0 && <p className="text-xs text-muted-foreground">{retained} remembered for returning to Selected templates; these IDs are not active in this mode.</p>}
  </div>;
}

const SessionPresentationRow = memo(function SessionPresentationRow({ draftId, disabled, id, search }: SessionControlProps & { id: string; search: string }) {
  const inputId = useId();
  const title = usePresentationField(id, "title");
  const enabled = usePresentationField(id, "enabled");
  const selected = useSessionPresentationMarked(draftId, id);
  const excluded = useSessionPresentationMarked(draftId, id, true);
  const mode = useSessionPresentationMode(draftId);
  const actions = useAgentSessionDraftActions();
  const matches = `${title ?? id} ${id}`.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase());
  if (!matches) return null;
  const unavailable = enabled !== true;
  const status = title === undefined ? "Unavailable — not in this catalog" : !enabled ? "Unavailable — disabled" : "Availability resolved at run admission";
  return <li className="flex flex-wrap items-center gap-x-2 gap-y-1 rounded-md bg-muted/40 px-3 py-2">
    <Label htmlFor={inputId} className="flex min-h-11 min-w-0 flex-1 cursor-pointer items-center gap-3 py-1">
      <Checkbox id={inputId} checked={selected} aria-describedby={`${inputId}-status`}
        disabled={disabled || mode !== "selected" || (unavailable && !selected)}
        onCheckedChange={() => actions.togglePresentation(draftId, id)} />
      <span className="min-w-0 break-words text-sm">{title ?? id}</span>
    </Label>
    <Button type="button" variant="secondary" className="min-h-11 shrink-0 text-xs"
      aria-label={`${excluded ? "Remove exclusion for" : "Exclude"} ${title ?? id}`}
      disabled={disabled || (unavailable && !excluded)}
      onClick={() => actions.togglePresentation(draftId, id, true)}>{excluded ? "Remove exclusion" : "Exclude"}</Button>
    <p id={`${inputId}-status`} className="w-full break-words text-xs text-muted-foreground">{status}{excluded ? "; excluded here in every mode" : ""}</p>
  </li>;
});

function SessionPresentationChoices({ draftId, disabled }: SessionControlProps) {
  const searchId = useId();
  const [search, setSearch] = useState("");
  const catalogIds = usePresentationIds();
  const selected = useSessionPresentationIds(draftId);
  const excluded = useSessionPresentationIds(draftId, true);
  const mode = useSessionPresentationMode(draftId);
  const matches = useSessionPresentationMatchCount(draftId, search);
  const ids = [...new Set([...catalogIds, ...selected, ...excluded])];
  const actions = useAgentSessionDraftActions();
  return <div className="space-y-3">
    <p className="text-xs text-muted-foreground" aria-live="polite">
      {mode === "selected" ? `${selected.length} requested; ` : ""}{excluded.length} exclusions apply in every mode.
      {mode === "selected" && selected.length === 0 ? " No templates are requested." : ""}
    </p>
    {excluded.length > 0 && <ul aria-label="Active exclusions" className="space-y-1">
      {excluded.map((id) => <SessionExclusion key={id} id={id} draftId={draftId} disabled={disabled} />)}
    </ul>}
    <details open={mode === "selected"}>
      <summary className="min-h-11 cursor-pointer py-3 text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-ring">Browse templates and exclusions</summary>
      <div className="space-y-3 pt-2">
        <Label htmlFor={searchId}>Find a template</Label>
        <Input id={searchId} value={search} onChange={(event) => setSearch(event.target.value)} className="min-h-11" placeholder="Search title or ID" />
        {ids.length === 0 ? <p className="text-sm text-muted-foreground">No templates in this catalog. Create one to select it here.</p>
          : matches === 0 ? <div role="status"><p className="text-sm">No matching templates.</p><Button type="button" variant="secondary" className="mt-2 min-h-11" onClick={() => setSearch("")}>Clear search</Button></div>
            : <ul aria-label="Template choices" className="max-h-80 space-y-2 overflow-y-auto">{ids.map((id) => <SessionPresentationRow key={id} id={id} draftId={draftId} disabled={disabled} search={search} />)}</ul>}
        <a href="/admin/presentations" target="_blank" rel="noopener noreferrer" className="inline-flex min-h-11 items-center text-sm text-primary underline underline-offset-4 focus-visible:outline focus-visible:outline-2 focus-visible:outline-ring">Manage templates (opens in a new tab)</a>
      </div>
    </details>
    <Button type="button" variant="secondary" disabled={disabled} className="min-h-11" onClick={() => actions.resetPresentations(draftId)}>Reset assignment</Button>
    <p className="text-xs text-muted-foreground">Reset restores inheritance and clears selections and exclusions. Save Configuration applies these changes.</p>
  </div>;
}

const SessionExclusion = memo(function SessionExclusion({ id, draftId, disabled }: SessionControlProps & { id: string }) {
  const title = usePresentationField(id, "title");
  const actions = useAgentSessionDraftActions();
  return <li className="flex min-w-0 items-center gap-2 text-xs"><span className="min-w-0 flex-1 break-words">Excluded: {title ?? id}</span>
    <Button type="button" variant="secondary" disabled={disabled} className="min-h-11 shrink-0 text-xs" aria-label={`Remove exclusion for ${title ?? id}`} onClick={() => actions.togglePresentation(draftId, id, true)}>Remove</Button></li>;
});

export function SessionPresentationSelection({ draftId, disabled }: SessionControlProps) {
  const headingId = useId();
  const status = usePresentationCatalogField("status");
  const catalogError = usePresentationCatalogField("error");
  const assignmentError = useSessionPresentationError(draftId);
  const ready = useSessionPresentationReady(draftId);
  const draftExists = useAgentSessionDraftStatus(draftId) !== null;
  const catalogActions = usePresentationActions();
  const actions = useAgentSessionDraftActions();
  const admit = () => actions.admitPresentations(draftId);
  useEffect(() => { void catalogActions.ensureLoaded(); }, [catalogActions]);
  useEffect(() => {
    if (status === "ready" && draftExists) void actions.admitPresentations(draftId);
  }, [actions, draftExists, draftId, status]);
  return <section aria-labelledby={headingId} className="space-y-4">
    <div className="space-y-1"><h3 id={headingId} className="text-sm font-semibold">Presentations</h3>
      <p className="text-xs text-muted-foreground">Reusable UI templates for future runs. Parent policy and client support can narrow this request; saving does not render a template.</p></div>
    {!ready ? <div className="space-y-2" role="status">
      <p className="text-sm text-muted-foreground">{catalogError ?? assignmentError ?? (status === "ready" ? "Loading the current assignment." : "Assignment is unavailable until the catalog verifies its owner. Your draft is retained.")}</p>
      <Button type="button" variant="secondary" className="min-h-11" disabled={status === "loading" || disabled}
        onClick={() => { if (status === "ready") void admit(); else void catalogActions.reload(); }}>{status === "ready" ? "Reload assignment" : "Reload catalog"}</Button>
      <p className="text-xs text-muted-foreground">Other settings can still be saved if you have no unsaved Presentation changes.</p>
    </div> : <><SessionPresentationMode draftId={draftId} disabled={disabled} /><SessionPresentationChoices draftId={draftId} disabled={disabled} />
      <Button type="button" variant="secondary" className="min-h-11" disabled={disabled} onClick={() => { void catalogActions.reload(); }}>Reload catalog</Button></>}
  </section>;
}
