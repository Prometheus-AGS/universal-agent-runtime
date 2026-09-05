import { useEffect, useRef, useState } from "react";
import { ChevronRight, Plus, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  usePresentationActions, usePresentationCatalogField, usePresentationDraftField,
  usePresentationField, usePresentationIds, usePresentationMatchCount,
} from "@/platform/entities";
import { PresentationConfirmation } from "./presentation-confirmation";
import { PresentationEditor } from "./presentation-editor";

function RowDescription({ id }: { id: string }) {
  const description = usePresentationField(id, "description");
  return description ? <span className="line-clamp-2 break-words text-sm text-muted-foreground">{description}</span> : null;
}

function RowStatus({ id }: { id: string }) {
  const enabled = usePresentationField(id, "enabled");
  const revision = usePresentationField(id, "revision");
  return <span className="flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground"><span>{enabled ? "Available" : "Disabled"}</span><span>Revision {revision}</span></span>;
}

function PresentationRow({ id, search, onOpen }: { id: string; search: string; onOpen: (id?: string) => void }) {
  const title = usePresentationField(id, "title");
  const dirty = usePresentationDraftField("dirty");
  if (title === undefined || !title.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase())) return null;
  return (
    <li>
      <button id={`presentation-row-${id}`} type="button" disabled={dirty}
        className="flex min-h-11 w-full items-center gap-4 rounded-md p-4 text-left transition-colors hover:bg-muted focus-visible:outline-2 focus-visible:outline-ring disabled:opacity-50"
        onClick={() => onOpen(id)} aria-describedby={dirty ? "presentation-recovery-hint" : undefined}>
        <span className="flex min-w-0 flex-1 flex-col gap-2"><span className="break-words font-medium">{title}</span><RowDescription id={id} /><RowStatus id={id} /></span>
        <ChevronRight aria-hidden="true" className="size-4 shrink-0 text-muted-foreground" />
      </button>
    </li>
  );
}

function RecoveryBanner() {
  const dirty = usePresentationDraftField("dirty");
  const title = usePresentationDraftField("title");
  const actions = usePresentationActions();
  const [confirmDiscard, setConfirmDiscard] = useState(false);
  if (!dirty) return null;
  return (
    <section aria-labelledby="presentation-recovery-heading" className="space-y-3 rounded-md bg-muted p-4">
      <div>
        <h2 id="presentation-recovery-heading" className="font-medium">Continue your unsaved draft</h2>
        <p id="presentation-recovery-hint" className="mt-1 break-words text-sm text-muted-foreground">“{title || "Untitled Presentation"}” has unsaved edits. Resume or discard this draft before opening another template.</p>
      </div>
      <div className="flex flex-wrap gap-2">
        <Button type="button" className="min-h-11" onClick={() => actions.resume()}>Resume draft</Button>
        <Button type="button" variant="ghost" className="min-h-11" onClick={() => setConfirmDiscard(true)}>Discard draft</Button>
      </div>
      <PresentationConfirmation open={confirmDiscard} title="Discard this draft?" description="Your local edits will be removed. Saved Presentations will not change."
        confirmLabel="Discard draft" onOpenChange={setConfirmDiscard} onConfirm={() => { actions.close(true); setConfirmDiscard(false); }} />
    </section>
  );
}

function Registry({ onOpen }: { onOpen: (id?: string) => void }) {
  const status = usePresentationCatalogField("status");
  const error = usePresentationCatalogField("error");
  const actions = usePresentationActions();
  const ids = usePresentationIds();
  const [search, setSearch] = useState("");
  const matches = usePresentationMatchCount(search);
  if (!status || status === "loading") return (
    <div role="status" className="space-y-3 py-6"><p className="text-sm text-muted-foreground">Loading your Presentations…</p><div className="h-20 rounded-md bg-muted motion-safe:animate-pulse" aria-hidden="true" /></div>
  );
  if (status === "error") return (
    <div className="space-y-4 rounded-md bg-muted p-6"><p role="alert" className="text-sm text-destructive">{error}</p><Button type="button" variant="secondary" className="min-h-11" onClick={() => void actions.reload()}><RefreshCw aria-hidden="true" />Reload catalog</Button></div>
  );
  return (
    <div className="space-y-6">
      <RecoveryBanner />
      {ids.length === 0 ? (
        <section className="space-y-3 rounded-lg bg-muted/40 px-6 py-10">
          <h2 className="text-lg font-medium">Build a reusable starting point</h2>
          <p className="max-w-prose text-sm text-muted-foreground">No Presentations yet. Create a template for the information your agents display, preview it safely, then save a revision. Assignment is configured separately.</p>
          <NewPresentationButton onOpen={onOpen} id="presentation-empty-new" />
        </section>
      ) : (
        <>
          <div className="flex flex-wrap items-end gap-3">
            <div className="min-w-0 flex-1 space-y-2"><Label htmlFor="presentation-search">Find a Presentation</Label><Input id="presentation-search" type="search" className="min-h-11" value={search} placeholder="Search by title" onChange={(event) => setSearch(event.target.value)} /></div>
            <Button type="button" variant="secondary" className="min-h-11" onClick={() => void actions.reload()}><RefreshCw aria-hidden="true" />Reload</Button>
          </div>
          <p role="status" className="text-sm text-muted-foreground">{matches} {matches === 1 ? "Presentation" : "Presentations"}{search.trim() ? " matching your search" : " in your catalog"}</p>
          {matches === 0 ? <div className="space-y-3 py-6"><p>No titles match “{search}”.</p><Button type="button" variant="secondary" className="min-h-11" onClick={() => setSearch("")}>Clear search</Button></div>
            : <ul aria-label="Presentations" className="space-y-1">{ids.map((id) => <PresentationRow key={id} id={id} search={search} onOpen={onOpen} />)}</ul>}
        </>
      )}
    </div>
  );
}

function NewPresentationButton({ onOpen, id = "presentation-new" }: { onOpen: () => void; id?: string }) {
  const status = usePresentationCatalogField("status");
  const dirty = usePresentationDraftField("dirty");
  return <Button id={id} type="button" className="min-h-11" disabled={status !== "ready" || dirty} onClick={() => onOpen()}><Plus aria-hidden="true" />New Presentation</Button>;
}

export function PresentationsPage() {
  const actions = usePresentationActions();
  const editorOpen = usePresentationCatalogField("editor_open");
  const focusTarget = useRef("presentation-new");
  useEffect(() => { void actions.ensureLoaded(); }, [actions]);
  const open = (id?: string) => {
    focusTarget.current = id ? `presentation-row-${id}` : "presentation-new";
    actions.begin(id);
  };
  const restoreFocus = (deleted = false) => {
    requestAnimationFrame(() => {
      const target = document.getElementById(deleted ? "presentations-heading" : focusTarget.current);
      const usableTarget = target instanceof HTMLButtonElement && target.disabled ? null : target;
      (usableTarget ?? document.getElementById("presentations-heading"))?.focus();
    });
  };
  return (
    <div className="min-w-0 flex-1 overflow-y-auto p-4 sm:p-6">
      <div className="mx-auto w-full max-w-7xl space-y-6">
        {editorOpen ? <PresentationEditor onClosed={restoreFocus} /> : (
          <>
            <header className="flex flex-wrap items-start justify-between gap-4">
              <div className="min-w-0 space-y-2"><h1 id="presentations-heading" tabIndex={-1} className="text-xl font-semibold focus-visible:outline-2 focus-visible:outline-ring">Presentations</h1><p className="max-w-prose text-sm text-muted-foreground">Reusable UI templates for your agents. Manage content here; assign eligibility separately.</p></div>
              <NewPresentationButton onOpen={open} />
            </header>
            <Registry onOpen={open} />
          </>
        )}
      </div>
    </div>
  );
}
