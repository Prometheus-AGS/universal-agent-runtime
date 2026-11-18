import { useId, useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { workspaceProblem } from "../model/teams-location";

/**
 * The workspace is URL state, committed on Enter. The draft lives here; the parent remounts this control
 * (key = committed workspace) so the draft always starts from the committed value.
 */
export function WorkspaceControl({ workspace, onCommit }: { workspace: string; onCommit: (workspace: string) => void }) {
  const id = useId();
  const [draft, setDraft] = useState(workspace);
  const [problem, setProblem] = useState<string | null>(null);
  const submit = (event: FormEvent) => {
    event.preventDefault();
    const text = draft.trim();
    const found = workspaceProblem(text);
    setProblem(found);
    if (!found && text !== workspace) onCommit(text);
  };
  const describedBy = problem ? `${id}-problem` : `${id}-hint`;
  return (
    <form onSubmit={submit} className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1" noValidate>
      <Label htmlFor={id} className="shrink-0">Workspace</Label>
      <Input id={id} className="min-h-11 w-44 min-w-0 flex-1 md:min-h-9 md:w-52 md:flex-none" value={draft} spellCheck={false} autoComplete="off"
        aria-invalid={problem ? true : undefined} aria-describedby={describedBy}
        onChange={(event) => { setDraft(event.target.value); setProblem(null); }} />
      <Button type="submit" variant="secondary" className="min-h-11 md:min-h-9">Switch</Button>
      {problem ? (
        <p id={`${id}-problem`} role="alert" className="w-full text-xs text-destructive">{problem}</p>
      ) : (
        <p id={`${id}-hint`} className="sr-only">Press Enter or choose Switch. Bindings and teams belong to one workspace.</p>
      )}
    </form>
  );
}
