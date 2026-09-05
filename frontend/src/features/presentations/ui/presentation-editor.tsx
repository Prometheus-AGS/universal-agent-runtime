import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowLeft, Save, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { parsePresentationSource, usePresentationActions, usePresentationDraftField } from "@/platform/entities";
import { PresentationConfirmation } from "./presentation-confirmation";
import { PresentationPreview } from "./presentation-preview";

function TitleField() {
  const value = usePresentationDraftField("title") ?? "";
  const status = usePresentationDraftField("status");
  const actions = usePresentationActions();
  const invalid = status === "error" && !value.trim();
  return (
    <div className="space-y-2">
      <Label htmlFor="presentation-title">Title <span className="text-muted-foreground">(required)</span></Label>
      <Input id="presentation-title" className="min-h-11" autoFocus value={value} disabled={status === "saving"}
        aria-invalid={invalid} aria-describedby={invalid ? "presentation-title-error" : undefined}
        onChange={(event) => actions.edit("title", event.target.value)} />
      {invalid && <p id="presentation-title-error" className="text-sm text-destructive">Enter a title.</p>}
    </div>
  );
}

function DescriptionField() {
  const value = usePresentationDraftField("description") ?? "";
  const saving = usePresentationDraftField("status") === "saving";
  const actions = usePresentationActions();
  return (
    <div className="space-y-2">
      <Label htmlFor="presentation-description">Description</Label>
      <Textarea id="presentation-description" value={value} disabled={saving} rows={3}
        aria-describedby="presentation-description-hint" onChange={(event) => actions.edit("description", event.target.value)} />
      <p id="presentation-description-hint" className="text-sm text-muted-foreground">Describe what this template presents and when an agent should use it.</p>
    </div>
  );
}

function AvailabilityField() {
  const value = usePresentationDraftField("enabled") ?? false;
  const saving = usePresentationDraftField("status") === "saving";
  const actions = usePresentationActions();
  return (
    <div className="flex min-h-11 items-center justify-between gap-4">
      <Label htmlFor="presentation-enabled" className="min-h-11 flex-1 cursor-pointer flex-col items-start justify-center gap-1">
        Available for future runs
        <span className="text-sm font-normal text-muted-foreground">Availability does not assign this template to an agent.</span>
      </Label>
      <Switch id="presentation-enabled" checked={value} disabled={saving} onCheckedChange={(checked) => actions.edit("enabled", checked)} />
    </div>
  );
}

function SourceField() {
  const source = usePresentationDraftField("source") ?? "";
  const saving = usePresentationDraftField("status") === "saving";
  const actions = usePresentationActions();
  const parsed = useMemo(() => parsePresentationSource(source), [source]);
  return (
    <div className="space-y-2">
      <Label htmlFor="presentation-source">Template source</Label>
      <p id="presentation-source-hint" className="text-sm text-muted-foreground">Declarative JSON, not executable code. Bind text to a data path such as <code className="font-mono">/message</code>.</p>
      <Textarea id="presentation-source" value={source} disabled={saving} rows={18} spellCheck={false}
        className="min-h-72 font-mono text-sm" aria-invalid={Boolean(parsed.error)}
        aria-describedby={`presentation-source-hint${parsed.error ? " presentation-source-error" : ""}`}
        onChange={(event) => actions.edit("source", event.target.value)} />
      {parsed.error && <p id="presentation-source-error" className="break-words text-sm text-destructive">{parsed.error}</p>}
      <details className="rounded-md bg-muted/40 p-3">
        <summary className="min-h-11 cursor-pointer content-center text-sm font-medium focus-visible:outline-2 focus-visible:outline-ring">Supported components and bindings</summary>
        <div className="space-y-3 pt-2 text-sm text-muted-foreground">
          <p>Text, Button, TextField, CheckBox, ChoicePicker, Row, Column, Card and Divider.</p>
          <p>Start with one component named <code className="font-mono">root</code>. Row and Column use <code className="font-mono">children</code>; Card and Button use <code className="font-mono">child</code>. Every other component needs exactly one parent.</p>
          <pre className="overflow-x-auto rounded-md bg-background p-3 font-mono text-xs">{'"text": { "path": "/message" }\n"default_data": { "message": "Ready" }'}</pre>
          <p>Button actions use an event name and optional context object. They are inert in this preview. Keep the starter version and catalog identifiers unchanged.</p>
        </div>
      </details>
    </div>
  );
}

function SaveFeedback() {
  const error = usePresentationDraftField("error");
  const status = usePresentationDraftField("status");
  const dirty = usePresentationDraftField("dirty");
  const revision = usePresentationDraftField("expected_revision");
  const errorRef = useRef<HTMLDivElement>(null);
  useEffect(() => { if (error) errorRef.current?.focus(); }, [error]);
  return error ? (
    <div ref={errorRef} tabIndex={-1} role="alert" id="presentation-save-error"
      className="rounded-md bg-destructive/10 p-4 text-sm text-destructive focus-visible:outline-2 focus-visible:outline-ring">
      <p className="font-medium">The edit needs your attention</p><p className="mt-1 break-words">{error}</p>
    </div>
  ) : (
    <p role="status" className="text-sm text-muted-foreground">
      {status === "saving" ? "Saving… Keep this window open until the result is confirmed."
        : status === "saved" ? `Saved revision ${revision}.`
          : dirty ? "Unsaved changes. Your draft is retained in this browser." : revision ? `Editing revision ${revision}.` : "New template. Not saved yet."}
    </p>
  );
}

function EditorHeading() {
  const id = usePresentationDraftField("presentation_id");
  return <h1 className="text-xl font-semibold">{id ? "Edit Presentation" : "New Presentation"}</h1>;
}

function EditorActions({ onClosed }: { onClosed: (deleted?: boolean) => void }) {
  const actions = usePresentationActions();
  const saving = usePresentationDraftField("status") === "saving";
  const dirty = usePresentationDraftField("dirty");
  const id = usePresentationDraftField("presentation_id");
  const title = usePresentationDraftField("title") ?? "Untitled Presentation";
  const uncertain = usePresentationDraftField("uncertain");
  const conflict = usePresentationDraftField("conflict");
  const [confirmation, setConfirmation] = useState<"discard" | "delete" | "reload" | "retry" | null>(null);
  const close = () => {
    if (dirty) setConfirmation("discard");
    else { actions.close(false); onClosed(); }
  };
  const confirm = async () => {
    if (confirmation === "delete") {
      const deleted = await actions.remove();
      setConfirmation(null);
      if (deleted) onClosed(true);
    } else if (confirmation === "reload") {
      setConfirmation(null);
      await actions.reloadSavedVersion();
    } else if (confirmation === "retry") {
      actions.acknowledgeUncertainResult(); setConfirmation(null);
    } else {
      actions.close(true); setConfirmation(null); onClosed();
    }
  };
  return (
    <>
      <div className="flex flex-wrap items-center gap-2">
        <Button type="submit" className="min-h-11" disabled={saving || uncertain || conflict}><Save aria-hidden="true" />{saving ? "Saving…" : "Save Presentation"}</Button>
        <Button type="button" variant="secondary" className="min-h-11" disabled={saving} onClick={close}>Cancel</Button>
        {id && <Button type="button" variant="ghost" className="min-h-11 text-destructive sm:ml-auto" disabled={saving || uncertain || conflict} onClick={() => setConfirmation("delete")}><Trash2 aria-hidden="true" />Delete</Button>}
      </div>
      {conflict && <Button type="button" variant="secondary" className="min-h-11" onClick={() => setConfirmation("reload")}>Reload saved version</Button>}
      {uncertain && (
        <div className="space-y-2 rounded-md bg-muted p-4 text-sm">
          <p>The outcome is unconfirmed. Check the catalog before submitting this draft again; the first save may have succeeded.</p>
          <div className="flex flex-wrap gap-2">
            <Button type="button" variant="secondary" className="min-h-11" onClick={() => void actions.reload()}>Reload catalog</Button>
            <Button type="button" variant="secondary" className="min-h-11" onClick={() => setConfirmation("retry")}>I checked; allow another save</Button>
          </div>
        </div>
      )}
      <PresentationConfirmation open={confirmation !== null} pending={saving}
        title={confirmation === "delete" ? `Delete “${title}”?` : confirmation === "reload" ? "Replace this draft with the saved version?" : confirmation === "retry" ? "Allow another save?" : "Discard unsaved changes?"}
        description={confirmation === "delete" ? "This removes the template from future selection. Already-admitted runs retain their captured content. Any unsaved edits will also be discarded."
          : confirmation === "reload" ? "Your unsaved edits will be replaced only after the catalog loads successfully."
            : confirmation === "retry" ? "Continue only after checking whether the first request succeeded. Saving a new template again can create a second copy."
              : "This removes your local draft. The saved template will not change."}
        confirmLabel={confirmation === "delete" ? "Delete Presentation" : confirmation === "reload" ? "Reload and discard edits" : confirmation === "retry" ? "Allow another save" : "Discard changes"}
        onOpenChange={(open) => { if (!open) setConfirmation(null); }} onConfirm={() => void confirm()} />
    </>
  );
}

function BackButton({ onClosed }: { onClosed: () => void }) {
  const actions = usePresentationActions();
  const saving = usePresentationDraftField("status") === "saving";
  return <Button type="button" variant="ghost" className="min-h-11 self-start" disabled={saving} onClick={() => { actions.close(false); onClosed(); }}><ArrowLeft aria-hidden="true" />Back to catalog <span className="sr-only">(keep draft)</span></Button>;
}

export function PresentationEditor({ onClosed }: { onClosed: (deleted?: boolean) => void }) {
  const actions = usePresentationActions();
  return (
    <div className="space-y-6">
      <div className="space-y-3"><BackButton onClosed={onClosed} /><EditorHeading /></div>
      <SaveFeedback />
      <div className="grid items-start gap-6 xl:grid-cols-2">
        <form noValidate onSubmit={(event) => { event.preventDefault(); void actions.save(); }} className="min-w-0 space-y-6">
          <TitleField /><DescriptionField /><AvailabilityField /><SourceField />
          <EditorActions onClosed={onClosed} />
        </form>
        <PresentationPreview />
      </div>
    </div>
  );
}
