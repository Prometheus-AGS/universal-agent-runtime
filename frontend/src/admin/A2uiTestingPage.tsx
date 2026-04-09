import { type FC, useCallback, useEffect, useState } from "react";
import { FileJson, RefreshCw, Send } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { AdminEmptyInline, AdminError, AdminSidebarSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";

interface A2uiSchema {
  id: string;
  type: string;
  title?: string;
  description?: string;
  fields?: A2uiField[];
  options?: A2uiOption[];
  content?: string;
  [key: string]: unknown;
}

interface A2uiField {
  name: string;
  label?: string;
  type?: string;
  required?: boolean;
  placeholder?: string;
  options?: A2uiOption[];
}

interface A2uiOption {
  value: string;
  label: string;
}

export const A2uiTestingPage: FC = () => {
  const [schemas, setSchemas] = useState<A2uiSchema[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<A2uiSchema | null>(null);
  const [customJson, setCustomJson] = useState("");
  const [customError, setCustomError] = useState<string | null>(null);
  const [response, setResponse] = useState<Record<string, unknown> | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/uar/a2ui/schemas");
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = (await res.json()) as { schemas?: A2uiSchema[] } & A2uiSchema[];
      const list = Array.isArray(data) ? data : data.schemas ?? [];
      setSchemas(list);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const handlePreviewCustom = () => {
    setCustomError(null);
    try {
      const parsed = JSON.parse(customJson) as A2uiSchema;
      if (!parsed.type) throw new Error("Schema must have a 'type' field");
      setSelected(parsed);
      setResponse(null);
    } catch (e) {
      setCustomError((e as Error).message);
    }
  };

  const handleSubmit = (values: Record<string, unknown>) => {
    setResponse(values);
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">
            A2UI Schema Testing
          </h2>
          <p className="font-mono text-xs text-muted-foreground">
            {schemas.length} schemas available
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => void load()}
          className="gap-1.5"
        >
          <RefreshCw size={13} className={cn(loading && "animate-spin")} />
          Refresh
        </Button>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Left: schema list + custom JSON */}
        <div className="flex w-72 shrink-0 flex-col border-r border-border">
          <div className="flex-1 overflow-y-auto p-4">
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
                <button
                  key={schema.id}
                  onClick={() => {
                    setSelected(schema);
                    setResponse(null);
                  }}
                  className={cn(
                    "flex items-center gap-2 rounded-md px-3 py-2 text-left text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
                    selected?.id === schema.id
                      ? "bg-accent text-foreground"
                      : "text-muted-foreground hover:bg-muted/50 hover:text-foreground",
                  )}
                >
                  <FileJson
                    size={13}
                    className={cn(
                      selected?.id === schema.id
                        ? "text-primary"
                        : "text-muted-foreground",
                    )}
                  />
                  <div className="min-w-0 flex-1">
                    <p className="truncate font-mono text-xs font-medium">
                      {schema.id}
                    </p>
                    <p className="truncate font-mono text-xs text-muted-foreground/60">
                      {schema.type}
                    </p>
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Custom JSON input */}
          <div className="border-t border-border p-4">
            <p className="mb-2 font-mono text-xs uppercase tracking-widest text-muted-foreground">
              Custom Schema JSON
            </p>
            <Textarea
              value={customJson}
              onChange={(e) => setCustomJson(e.target.value)}
              placeholder='{"type":"form","id":"test",...}'
              className="mb-2 h-28 font-mono text-xs"
            />
            {customError && (
              <p className="mb-2 font-mono text-xs text-destructive">
                {customError}
              </p>
            )}
            <Button
              variant="outline"
              size="sm"
              onClick={handlePreviewCustom}
              className="w-full gap-1.5"
            >
              Preview
            </Button>
          </div>
        </div>

        {/* Right: preview + response */}
        <div className="flex flex-1 flex-col overflow-y-auto p-6">
          {!selected && (
            <div className="flex flex-1 items-center justify-center">
              <p className="font-mono text-xs text-muted-foreground">
                Select a schema or paste custom JSON to preview
              </p>
            </div>
          )}

          {selected && (
            <div className="space-y-6">
              {/* Schema info */}
              <div>
                <h3 className="font-display text-base font-semibold text-foreground">
                  {selected.title ?? selected.id}
                </h3>
                {selected.description && (
                  <p className="mt-1 text-sm text-muted-foreground">
                    {selected.description}
                  </p>
                )}
                <span className="mt-2 inline-block rounded border border-border px-2 py-0.5 font-mono text-xs text-muted-foreground">
                  {selected.type}
                </span>
              </div>

              {/* Preview renderer */}
              <SchemaPreview schema={selected} onSubmit={handleSubmit} />

              {/* Response JSON */}
              {response && (
                <div>
                  <p className="mb-1.5 font-mono text-xs uppercase tracking-widest text-muted-foreground">
                    Response Payload
                  </p>
                  <pre className="hljs rounded-md p-3 text-xs">
                    <code>{JSON.stringify(response, null, 2)}</code>
                  </pre>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

/* ---- Schema Preview Renderer ---- */

interface SchemaPreviewProps {
  schema: A2uiSchema;
  onSubmit: (values: Record<string, unknown>) => void;
}

const SchemaPreview: FC<SchemaPreviewProps> = ({ schema, onSubmit }) => {
  const [values, setValues] = useState<Record<string, string>>({});

  const handleChange = (name: string, value: string) => {
    setValues((prev) => ({ ...prev, [name]: value }));
  };

  switch (schema.type) {
    case "form":
      return (
        <div className="space-y-3 rounded-lg border border-border bg-card p-4">
          {schema.fields?.map((field) => (
            <div key={field.name}>
              <label className="mb-1 block font-mono text-xs font-medium text-foreground">
                {field.label ?? field.name}
                {field.required && (
                  <span className="ml-1 text-destructive">*</span>
                )}
              </label>
              {field.type === "select" && field.options ? (
                <select
                  value={values[field.name] ?? ""}
                  onChange={(e) => handleChange(field.name, e.target.value)}
                  className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm text-foreground"
                >
                  <option value="">Select...</option>
                  {field.options.map((opt) => (
                    <option key={opt.value} value={opt.value}>
                      {opt.label}
                    </option>
                  ))}
                </select>
              ) : field.type === "textarea" ? (
                <Textarea
                  value={values[field.name] ?? ""}
                  onChange={(e) => handleChange(field.name, e.target.value)}
                  placeholder={field.placeholder}
                  className="font-mono text-xs"
                />
              ) : (
                <Input
                  type={field.type ?? "text"}
                  value={values[field.name] ?? ""}
                  onChange={(e) => handleChange(field.name, e.target.value)}
                  placeholder={field.placeholder}
                  className="h-8 text-xs"
                />
              )}
            </div>
          ))}
          <Button
            size="sm"
            onClick={() => onSubmit(values)}
            className="gap-1.5"
          >
            <Send size={13} />
            Submit
          </Button>
        </div>
      );

    case "confirm":
      return (
        <div className="space-y-3 rounded-lg border border-border bg-card p-4">
          <p className="text-sm text-foreground">
            {schema.content ?? schema.description ?? "Do you want to proceed with this action?"}
          </p>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => onSubmit({ confirmed: false })}
            >
              Cancel
            </Button>
            <Button
              size="sm"
              onClick={() => onSubmit({ confirmed: true })}
            >
              Confirm
            </Button>
          </div>
        </div>
      );

    case "select":
      return (
        <div className="space-y-2 rounded-lg border border-border bg-card p-4">
          {schema.options?.map((opt) => (
            <button
              key={opt.value}
              onClick={() => onSubmit({ selected: opt.value })}
              className="flex w-full items-center gap-2 rounded-md border border-border/50 px-3 py-2.5 text-left text-sm text-foreground transition-colors hover:bg-muted/50"
            >
              {opt.label}
            </button>
          ))}
        </div>
      );

    case "text-input":
      return (
        <div className="space-y-3 rounded-lg border border-border bg-card p-4">
          <Input
            value={values._text ?? ""}
            onChange={(e) => handleChange("_text", e.target.value)}
            placeholder={schema.description ?? "Enter text..."}
            className="h-8 text-xs"
          />
          <Button
            size="sm"
            onClick={() => onSubmit({ text: values._text ?? "" })}
            className="gap-1.5"
          >
            <Send size={13} />
            Send
          </Button>
        </div>
      );

    case "display":
      return (
        <div className="rounded-lg border border-border bg-card p-4">
          <p className="text-sm text-foreground">
            {schema.content ?? JSON.stringify(schema, null, 2)}
          </p>
        </div>
      );

    default:
      return (
        <div className="rounded-lg border border-border bg-card p-4">
          <p className="mb-2 font-mono text-xs text-muted-foreground">
            Unknown schema type: {schema.type}
          </p>
          <pre className="hljs rounded-md p-3 text-xs">
            <code>{JSON.stringify(schema, null, 2)}</code>
          </pre>
        </div>
      );
  }
};
