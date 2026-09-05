import { Button } from "@/components/ui/button";
import {
  usePresentationProvenanceActions,
  usePresentationProvenanceField,
  usePresentationProvenanceStatus,
  usePresentationProvenanceSubscription,
  type PresentationObservation,
} from "@/platform/entities";

const MODE_LABELS: Record<PresentationObservation["effective_mode"], string> = {
  legacy: "Legacy (not negotiated)",
  auto: "Text or eligible UI",
  text: "Text only",
  a2ui: "UI with a text summary",
  hybrid: "Text and UI",
};
const REQUEST_LABELS: Record<NonNullable<PresentationObservation["requested_mode"]>, string> = {
  auto: "Automatic", text: "Text", a2ui: "A2UI", hybrid: "Text and UI",
};
const OUTCOME_LABELS: Record<PresentationObservation["run_outcome"], string> = {
  running: "Running", finished: "Finished", failed: "Failed", cancelled: "Cancelled",
};
const FALLBACK_LABELS: Record<NonNullable<PresentationObservation["fallback_reason"]>, string> = {
  client_rendering_not_declared: "Text fallback: the client did not declare UI rendering support.",
  incompatible_profile: "Text fallback: the client does not support the required UI profile.",
  no_eligible_templates: "Text fallback: no eligible template was available for this run.",
  parent_text_ceiling: "Text fallback: the parent run permits text only.",
  no_surface_published: "The run finished without publishing a generated UI surface.",
  surface_generation_failed: "The run finished without a generated UI surface after generation failed.",
};

function RequestedOutput({ runId }: { runId: string }) {
  const requested = usePresentationProvenanceField(runId, "requested_mode");
  const effective = usePresentationProvenanceField(runId, "effective_mode");
  return <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(0,1.5fr)] gap-x-3">
    <dt className="text-fg-sub">Requested</dt>
    <dd className="min-w-0 break-words">{requested ? REQUEST_LABELS[requested]
      : effective === "legacy" ? "Legacy (not negotiated)" : "Not negotiated"}</dd>
  </div>;
}

function AdmittedOutput({ runId }: { runId: string }) {
  const mode = usePresentationProvenanceField(runId, "effective_mode");
  return <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(0,1.5fr)] gap-x-3">
    <dt className="text-fg-sub">Admitted output</dt>
    <dd className="min-w-0 break-words">{mode ? MODE_LABELS[mode] : "Unknown"}</dd>
  </div>;
}

function RunOutcome({ runId }: { runId: string }) {
  const outcome = usePresentationProvenanceField(runId, "run_outcome");
  return <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(0,1.5fr)] gap-x-3">
    <dt className="text-fg-sub">Run outcome</dt>
    <dd className="min-w-0 break-words">{outcome ? OUTCOME_LABELS[outcome] : "Unknown"}</dd>
  </div>;
}

function FallbackDetails({ runId }: { runId: string }) {
  const reason = usePresentationProvenanceField(runId, "fallback_reason");
  const failed = usePresentationProvenanceField(runId, "generation_failed");
  return <>
    {reason && <p>{FALLBACK_LABELS[reason]}</p>}
    {failed && reason !== "surface_generation_failed" && <p>A UI generation attempt failed.</p>}
  </>;
}

function SurfacePublication({ runId }: { runId: string }) {
  const published = usePresentationProvenanceField(runId, "surface_published");
  const outcome = usePresentationProvenanceField(runId, "run_outcome");
  return <>
    <p>{published ? "Generated UI surface published."
      : outcome === "running" ? "No generated UI surface published yet." : "No generated UI surface published."}</p>
    <p>Policy summaries do not count as generated UI surfaces.</p>
  </>;
}

function PublishedTemplates({ runId }: { runId: string }) {
  const templates = usePresentationProvenanceField(runId, "published_templates");
  const receipts = usePresentationProvenanceField(runId, "receipt_status");
  const outcome = usePresentationProvenanceField(runId, "run_outcome");
  if (receipts !== "available" || templates === undefined) return <p>Template publication receipts are unavailable for this run.</p>;
  if (templates.length === 0) return <p>{outcome === "running" ? "No template published yet." : "No template published."}</p>;
  return <details className="min-w-0">
    <summary className="min-h-11 cursor-pointer content-center rounded-sm py-2 text-foreground underline-offset-4 hover:underline focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember">
      Published templates ({templates.length})
    </summary>
    <ul className="mt-2 space-y-3">
      {templates.map((template) => <li key={template.template_id} className="min-w-0">
        <p className="break-all font-mono text-foreground">{template.template_id}</p>
        <p className="text-fg-sub">Revision <span className="font-mono">{template.revision}</span></p>
      </li>)}
    </ul>
  </details>;
}

function PresentationDetailsState({ runId }: { runId: string }) {
  const status = usePresentationProvenanceStatus(runId);
  const actions = usePresentationProvenanceActions();
  if (status === "loading" || status === "idle") return <p role="status">Loading Presentation details…</p>;
  if (status === "missing") return <p>Presentation details were not recorded for this run.</p>;
  if (status === "unsupported") return <p>This client cannot read the recorded Presentation details. No publication outcome can be confirmed here.</p>;
  if (status === "error") return <div className="space-y-2">
    <p>Presentation details could not be loaded from local run history.</p>
    <Button type="button" variant="ghost" className="min-h-11 focus-visible:ring-0 focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember" onClick={() => actions.retry(runId)}>
      Retry details
    </Button>
  </div>;
  return <div className="min-w-0 space-y-3">
    <dl className="space-y-2 text-foreground">
      <RequestedOutput runId={runId} />
      <AdmittedOutput runId={runId} />
      <RunOutcome runId={runId} />
    </dl>
    <p>Output permitted by the host; publication is recorded below.</p>
    <FallbackDetails runId={runId} />
    <SurfacePublication runId={runId} />
    <PublishedTemplates runId={runId} />
    <p>Publication does not confirm client display.</p>
  </div>;
}

export function PresentationRunDetails({ runId }: { runId: string }) {
  usePresentationProvenanceSubscription(runId);
  return <section aria-label="Presentation" className="min-w-0 space-y-3 text-sm leading-relaxed text-fg-sub">
    <div className="space-y-1">
      <h3 className="font-semibold text-foreground">Presentation</h3>
      <p>Latest recorded details for this run.</p>
    </div>
    <PresentationDetailsState runId={runId} />
  </section>;
}
