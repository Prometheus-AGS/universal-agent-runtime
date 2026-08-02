import { type FC, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router";
import { ExternalLink, FileJson, Loader2, RefreshCw, Zap } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { AdminEmptyInline, AdminError, AdminSidebarSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";
import { useA2uiSchemas } from "@/hooks/use-a2ui-schemas";
import { useGraphEntities } from "@/entities/hooks/use-graph-entities";
import type { RuntimeRunEntity } from "@/entities/types";
import type { A2uiArtifactSchema, A2uiComponent } from "@/features/a2ui/a2ui-protocol";
import { A2uiSurfaceRenderer } from "@/features/a2ui/a2ui-surface-renderer";
import { useThreadRegistryActions } from "@/hooks/use-thread-registry-actions";

/** Sensible example content per builtin artifact_type, pre-filled when a schema is selected. */
const EXAMPLE_CONTENT: Record<string, string> = {
  confirm: JSON.stringify({ message: "Test confirmation prompt", accept_label: "Yes", cancel_label: "No" }, null, 2),
  select: JSON.stringify({ prompt: "Choose one", options: [{ value: "a", label: "Option A" }, { value: "b", label: "Option B" }] }, null, 2),
  text_input: JSON.stringify({ prompt: "Enter some test input", placeholder: "Type here...", multiline: false }, null, 2),
  form: JSON.stringify({ title: "Test form", fields: { name: { type: "string", description: "Your name" } } }, null, 2),
};

const ACTIVE_RUN_STATUSES = new Set(["running", "waiting"]);

function previewSurface(title: string, content: string): A2uiComponent[] {
  return [
    { id: "title", component: "Text", text: title, variant: "h2" },
    { id: "content", component: "Text", text: content || "The progressive surface preview will appear here." },
    { id: "action-label", component: "Text", text: "Preview action" },
    {
      id: "action",
      component: "Button",
      child: "action-label",
      variant: "primary",
      action: { event: { name: "preview" } },
    },
    { id: "root", component: "Column", children: ["title", "content", "action"] },
  ];
}

export const A2uiTestingPage: FC = () => {
  const navigate = useNavigate();
  const { schemas, loading, error, load, triggering, triggerError, trigger } = useA2uiSchemas();
  const runs = useGraphEntities<RuntimeRunEntity>("RuntimeRun");
  const { setActiveThread } = useThreadRegistryActions();

  const activeRuns = useMemo(
    () => runs.filter((r) => ACTIVE_RUN_STATUSES.has(r.status)),
    [runs],
  );

  const [selectedSchema, setSelectedSchema] = useState<A2uiArtifactSchema | null>(null);
  const [selectedRunId, setSelectedRunId] = useState<string>("");
  const [content, setContent] = useState("");
  const [triggeredThreadId, setTriggeredThreadId] = useState<string | null>(null);
  const [previewStatus, setPreviewStatus] = useState<string | null>(null);

  useEffect(() => {
    if (schemas.length > 0 && !selectedSchema) setSelectedSchema(schemas[0] ?? null);
  }, [schemas, selectedSchema]);

  useEffect(() => {
    if (selectedSchema) {
      setContent(EXAMPLE_CONTENT[selectedSchema.artifact_type] ?? "{}");
      setTriggeredThreadId(null);
      setPreviewStatus(null);
    }
  }, [selectedSchema]);

  const selectedRun = activeRuns.find((r) => r.id === selectedRunId);

  const handleTrigger = async () => {
    if (!selectedSchema || !selectedRun) return;
    let metadata: Record<string, unknown> = {};
    try {
      const parsed = JSON.parse(content) as unknown;
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) metadata = parsed as Record<string, unknown>;
    } catch {
      // Display content may be plain text.
    }
    const succeeded = await trigger(selectedRun.id, {
      artifact_type: selectedSchema.artifact_type,
      title: selectedSchema.title,
      content,
      metadata,
    });
    if (succeeded) {
      setTriggeredThreadId(selectedRun.thread_id ?? null);
    }
  };

  const handleGoToThread = () => {
    if (!triggeredThreadId) return;
    setActiveThread(triggeredThreadId);
    navigate("/threads");
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">
            A2UI Live Testing
          </h2>
          <p className="font-mono text-xs text-muted-foreground">
            {schemas.length} schemas · {activeRuns.length} active run(s)
          </p>
        </div>
        <Button type="button" variant="outline" size="sm" onClick={() => void load()} className="gap-1.5">
          <RefreshCw size={13} className={cn(loading && "animate-spin")} />
          Refresh
        </Button>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Left: schema list */}
        <div className="flex w-72 shrink-0 flex-col overflow-y-auto border-r border-border p-4">
          <p className="mb-2 font-mono text-xs uppercase tracking-widest text-muted-foreground">
            Schemas
          </p>
          {loading && schemas.length === 0 && <AdminSidebarSkeleton rows={4} />}
          <AdminError error={error} />
          {!loading && schemas.length === 0 && !error && (
            <AdminEmptyInline>No schemas found</AdminEmptyInline>
          )}
          <div className="flex flex-col gap-1">
            {schemas.map((schema) => (
              <Button
                key={schema.schema_id}
                type="button"
                variant="ghost"
                onClick={() => setSelectedSchema(schema)}
                className={cn(
                  "h-auto w-full justify-start gap-2 px-3 py-2 text-left text-sm font-normal",
                  selectedSchema?.schema_id === schema.schema_id
                    ? "bg-accent text-foreground hover:bg-accent"
                    : "text-muted-foreground hover:bg-muted/50 hover:text-foreground",
                )}
              >
                <FileJson
                  size={13}
                  className={selectedSchema?.schema_id === schema.schema_id ? "text-primary" : "text-muted-foreground"}
                />
                <div className="min-w-0 flex-1 text-left">
                  <p className="truncate font-mono text-xs font-medium">{schema.title}</p>
                  <p className="truncate font-mono text-xs text-muted-foreground/60">
                    {schema.schema_id} · {schema.artifact_type}
                  </p>
                </div>
              </Button>
            ))}
          </div>
        </div>

        {/* Right: trigger form */}
        <div className="flex flex-1 flex-col overflow-y-auto p-6">
          {!selectedSchema ? (
            <div className="flex flex-1 items-center justify-center">
              <p className="font-mono text-xs text-muted-foreground">Select a schema to test</p>
            </div>
          ) : (
            <div className="max-w-xl space-y-5">
              <div>
                <h3 className="font-display text-base font-semibold text-foreground">
                  {selectedSchema.title}
                </h3>
                {selectedSchema.description && (
                  <p className="mt-1 text-sm text-muted-foreground">{selectedSchema.description}</p>
                )}
                <span className="mt-2 inline-block rounded border border-border px-2 py-0.5 font-mono text-xs text-muted-foreground">
                  {selectedSchema.artifact_type}
                </span>
              </div>

              <div>
                <p className="mb-1.5 font-mono text-xs uppercase tracking-widest text-muted-foreground">
                  Target run
                </p>
                {activeRuns.length === 0 ? (
                  <AdminEmptyInline>
                    No active runs right now — start a chat conversation first, then return here to trigger a test artifact against it.
                  </AdminEmptyInline>
                ) : (
                  <Select value={selectedRunId || undefined} onValueChange={(v) => v != null && setSelectedRunId(v)}>
                    <SelectTrigger className="h-9 w-full">
                      <SelectValue placeholder="Choose an active run..." />
                    </SelectTrigger>
                    <SelectContent>
                      {activeRuns.map((r) => (
                        <SelectItem key={r.id} value={r.id}>
                          {r.title ?? r.id} ({r.status})
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
              </div>

              <div>
                <p className="mb-1.5 font-mono text-xs uppercase tracking-widest text-muted-foreground">
                  Content / metadata JSON
                </p>
                <Textarea
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
                  rows={6}
                  className="font-mono text-xs"
                />
              </div>

              <section className="space-y-2" aria-label="A2UI surface preview">
                <h4 className="text-sm font-medium text-foreground">Shared React renderer preview</h4>
                <div className="rounded-lg border border-border bg-card p-4">
                  <A2uiSurfaceRenderer
                    components={previewSurface(selectedSchema.title, content)}
                    data={{}}
                    onDataChange={() => undefined}
                    onAction={() => setPreviewStatus("Preview action captured. Live runs submit through the A2UI action service.")}
                    statusMessage={previewStatus}
                  />
                </div>
              </section>

              <Button
                type="button"
                size="sm"
                disabled={!selectedRun || triggering}
                onClick={() => void handleTrigger()}
                className="gap-1.5"
              >
                {triggering ? <Loader2 size={13} className="animate-spin" /> : <Zap size={13} />}
                Trigger Test Artifact
              </Button>

              {triggerError && <AdminError error={triggerError} />}

              {triggeredThreadId && (
                <div className="flex items-center gap-2 rounded-md border border-primary/25 bg-primary/5 px-3 py-2.5">
                  <p className="flex-1 font-body text-sm text-foreground">
                    Triggered. Open the thread to interact with the real A2UI input block.
                  </p>
                  <Button type="button" size="sm" variant="outline" onClick={handleGoToThread} className="gap-1.5">
                    <ExternalLink size={13} />
                    Go to thread
                  </Button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
