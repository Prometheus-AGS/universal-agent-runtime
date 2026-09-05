import { useMemo, useState } from "react";
import { A2uiSurfaceRenderer } from "@/features/a2ui";
import { parsePresentationSource, usePresentationDraftField, type PresentationTemplate } from "@/platform/entities";

function TemplatePreview({ template }: { template: PresentationTemplate }) {
  // This is an inert preview sandbox, not a duplicate persisted business record.
  const [data, setData] = useState<Record<string, unknown>>(() => structuredClone(template.default_data));
  const [message, setMessage] = useState<string | null>(null);
  const change = (path: string, value: unknown) => {
    const parts = path.slice(1).split("/").map((part) => part.replaceAll("~1", "/").replaceAll("~0", "~"));
    const copy = structuredClone(data);
    let target = copy;
    for (const [index, part] of parts.entries()) {
      // `/items/length` is readable, but must not assign an array's length.
      if (Array.isArray(target) && (!/^(0|[1-9]\d*)$/.test(part) || Number(part) > target.length)) {
        setMessage("This preview binding is read-only. Array edits require an existing index or the next index.");
        return;
      }
      if (index === parts.length - 1) target[part] = value;
      else {
        if (!target[part] || typeof target[part] !== "object") target[part] = {};
        target = target[part] as Record<string, unknown>;
      }
    }
    setData(copy);
    setMessage(null);
  };
  return (
    <A2uiSurfaceRenderer
      components={template.components}
      data={data}
      idPrefix="presentation-preview-"
      onDataChange={change}
      onAction={(name) => setMessage(`Preview only: “${name}” was not sent to an agent or server.`)}
      statusMessage={message}
    />
  );
}

export function PresentationPreview() {
  const source = usePresentationDraftField("source") ?? "";
  const parsed = useMemo(() => parsePresentationSource(source), [source]);
  return (
    <section aria-labelledby="presentation-preview-heading" className="min-w-0 space-y-4 rounded-lg bg-muted/40 p-4 sm:p-6">
      <div className="space-y-1">
        <h2 id="presentation-preview-heading" className="text-base font-medium">Preview</h2>
        <p className="text-sm text-muted-foreground">Local preview. Actions do not run. Changes here are not saved as defaults.</p>
      </div>
      <div className="min-h-40 min-w-0 overflow-x-auto rounded-md bg-background p-4">
        {parsed.template
          ? <TemplatePreview key={source} template={parsed.template} />
          : <p className="text-sm text-muted-foreground">Preview unavailable. Fix the template source to see the current version.</p>}
      </div>
    </section>
  );
}
