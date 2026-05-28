import { type FC, useState } from "react";
import { ArrowLeft, CheckCircle2, Loader2, Play, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { JsonSchemaForm } from "@/components/json-schema-form";
import type { ToolGraphRow as ToolWithNs } from "@/entities/fetchers/tools";

interface ToolDetailPanelProps {
  tool: ToolWithNs;
  onBack?: () => void;
}

interface ExecuteResult {
  result: unknown;
  duration_ms: number;
  success: boolean;
  error?: string;
}

export const ToolDetailPanel: FC<ToolDetailPanelProps> = ({ tool, onBack }) => {
  const [args, setArgs] = useState<Record<string, unknown>>({});
  const [result, setResult] = useState<ExecuteResult | null>(null);
  const [executing, setExecuting] = useState(false);

  const schema = tool.input_schema ?? tool.parameters ?? {};

  const handleExecute = async () => {
    setExecuting(true);
    setResult(null);
    try {
      const res = await fetch(`/api/tools/${encodeURIComponent(tool._key)}/execute`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ arguments: args }),
      });
      const data = (await res.json()) as ExecuteResult;
      setResult(data);
    } catch (e) {
      setResult({
        result: null,
        duration_ms: 0,
        success: false,
        error: (e as Error).message,
      });
    } finally {
      setExecuting(false);
    }
  };

  const sourceLabel = tool.source === "mcp" && tool.server
    ? `MCP (${tool.server})`
    : tool.source === "built-in" || tool._ns === "built-in"
      ? "Built-in"
      : tool.source ?? "Unknown";

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="border-b border-border bg-card px-4 py-4 sm:px-6">
        <div className="flex items-center gap-2">
          {onBack && (
            <Button variant="ghost" size="icon" className="h-7 w-7 md:hidden" onClick={onBack} aria-label="Back to tools">
              <ArrowLeft size={14} />
            </Button>
          )}
          <div className="min-w-0 flex-1">
            <h2 className="font-display text-lg font-semibold text-foreground truncate">{tool._key}</h2>
            <p className="mt-0.5 font-mono text-xs text-muted-foreground">Source: {sourceLabel}</p>
          </div>
        </div>
        {tool.description && (
          <p className="mt-2 font-body text-xs leading-relaxed text-muted-foreground">{tool.description}</p>
        )}
      </div>

      {/* Tabs */}
      <div className="flex-1 overflow-y-auto px-4 py-4 sm:px-6">
        <Tabs defaultValue="test">
          <TabsList className="h-8">
            <TabsTrigger value="test" className="text-xs">Test</TabsTrigger>
            <TabsTrigger value="schema" className="text-xs">Schema</TabsTrigger>
          </TabsList>

          <TabsContent value="test" className="mt-4 flex flex-col gap-4">
            {/* Input section */}
            <div className="rounded-md border border-border p-4">
              <p className="mb-3 font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">Input</p>
              <JsonSchemaForm schema={schema} value={args} onChange={setArgs} />
              <div className="mt-4">
                <Button size="sm" onClick={() => void handleExecute()} disabled={executing} className="gap-1.5">
                  {executing ? <Loader2 size={13} className="animate-spin" /> : <Play size={13} />}
                  Execute
                </Button>
              </div>
            </div>

            {/* Result section */}
            {result && (
              <div className="rounded-md border border-border p-4">
                <div className="mb-3 flex items-center gap-2">
                  <p className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">Result</p>
                  {result.success ? (
                    <Badge variant="outline" className="gap-1 border-success/40 text-success text-xs">
                      <CheckCircle2 size={10} />Success
                    </Badge>
                  ) : (
                    <Badge variant="outline" className="gap-1 border-destructive/40 text-destructive text-xs">
                      <XCircle size={10} />Error
                    </Badge>
                  )}
                  {result.duration_ms > 0 && (
                    <span className="font-mono text-xs text-muted-foreground">
                      {(result.duration_ms / 1000).toFixed(2)}s
                    </span>
                  )}
                </div>
                {result.error && (
                  <pre className="mb-2 whitespace-pre-wrap rounded bg-destructive/10 p-3 font-mono text-xs text-destructive">
                    {result.error}
                  </pre>
                )}
                {result.result != null && (
                  <pre className="max-h-80 overflow-auto whitespace-pre-wrap rounded bg-muted/50 p-3 font-mono text-xs text-foreground">
                    {typeof result.result === "string" ? result.result : JSON.stringify(result.result, null, 2)}
                  </pre>
                )}
              </div>
            )}
          </TabsContent>

          <TabsContent value="schema" className="mt-4">
            <pre className="max-h-[600px] overflow-auto whitespace-pre-wrap rounded-md border border-border bg-muted/50 p-4 font-mono text-xs text-foreground">
              {JSON.stringify(schema, null, 2)}
            </pre>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
};
