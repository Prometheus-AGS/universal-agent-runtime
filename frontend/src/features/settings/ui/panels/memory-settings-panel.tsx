import { Input } from "@/components/ui/input";
import { NamespacePanel } from "../generic-schema-panel";
import { Field, SettingSelect, Toggle } from "../settings-primitives";

export function MemoryPanel() {
  return (
    <NamespacePanel
      namespace="memory"
      title="Agent Memory"
      subtitle="Persistent memory across conversations: auto-capture, context injection, and scoped retrieval."
      hint="Memory lets agents remember information across conversations. When enabled, agents can automatically save important details and recall them later. Scopes control who can access memories — 'user' scope means memories are private to each user, while 'global' is shared by all agents."
    >
      {({ val, set }) => (
        <>
          <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground/50 mb-2">
            Global Control
          </p>
          <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3 mb-1">
            <div>
              <p className="font-mono text-xs font-medium text-foreground">
                Enable Memory System
              </p>
              <p className="font-mono text-xs text-muted-foreground">
                Initialize the in-process vector memory store on startup.
              </p>
            </div>
            <Toggle
              value={(val("enabled") as boolean) ?? false}
              onChange={(v) => set("enabled", v)}
            />
          </div>

          <Field
            label="Activation Mode"
            hint="auto: capture+inject every turn. tool_only: LLM invokes memory tools. disabled: off."
          >
            <SettingSelect
              value={(val("activation_mode") as string) ?? "auto"}
              options={[
                { value: "auto", label: "auto (capture + inject)" },
                { value: "tool_only", label: "tool_only (LLM-driven)" },
                { value: "disabled", label: "disabled" },
              ]}
              onChange={(v) => set("activation_mode", v)}
            />
          </Field>

          <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground/50 mt-4 mb-2">
            Auto-Capture &amp; Injection
          </p>
          <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3 mb-1">
            <div>
              <p className="font-mono text-xs font-medium text-foreground">
                Auto-Capture
              </p>
              <p className="font-mono text-xs text-muted-foreground">
                Extract memories from each completed assistant turn.
              </p>
            </div>
            <Toggle
              value={(val("auto_capture") as boolean) ?? true}
              onChange={(v) => set("auto_capture", v)}
            />
          </div>
          <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3 mb-1">
            <div>
              <p className="font-mono text-xs font-medium text-foreground">
                Context Injection
              </p>
              <p className="font-mono text-xs text-muted-foreground">
                Inject relevant memories as a system prompt prefix before each
                LLM call.
              </p>
            </div>
            <Toggle
              value={(val("inject_context") as boolean) ?? true}
              onChange={(v) => set("inject_context", v)}
            />
          </div>
          <Field
            label="Max Context Tokens"
            hint="Maximum tokens to spend on injected memory context."
          >
            <Input
              type="number"
              min={100}
              max={32000}
              value={(val("max_context_tokens") as number) ?? 4096}
              onChange={(e) =>
                set("max_context_tokens", Number(e.target.value))
              }
              className="w-32 font-mono text-xs"
            />
          </Field>

          <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground/50 mt-4 mb-2">
            Retrieval Weights
          </p>
          <Field
            label="Vector (Semantic) Weight"
            hint="0.0–1.0. Higher = more semantic similarity."
          >
            <Input
              type="number"
              min={0}
              max={1}
              step={0.05}
              value={(val("vector_weight") as number) ?? 0.7}
              onChange={(e) => set("vector_weight", parseFloat(e.target.value))}
              className="w-28 font-mono text-xs"
            />
          </Field>
          <Field
            label="BM25 (Keyword) Weight"
            hint="0.0–1.0. Higher = more keyword match."
          >
            <Input
              type="number"
              min={0}
              max={1}
              step={0.05}
              value={(val("bm25_weight") as number) ?? 0.3}
              onChange={(e) => set("bm25_weight", parseFloat(e.target.value))}
              className="w-28 font-mono text-xs"
            />
          </Field>

          <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground/50 mt-4 mb-2">
            Embedding Provider
          </p>
          <Field label="Provider">
            <SettingSelect
              value={(val("embedding_provider") as string) ?? "openai"}
              options={[
                { value: "openai", label: "OpenAI" },
                { value: "cohere", label: "Cohere" },
                { value: "local", label: "Local" },
              ]}
              onChange={(v) => set("embedding_provider", v)}
            />
          </Field>
          <Field label="Embedding Model" hint="Provider-specific model name.">
            <Input
              value={
                (val("embedding_model") as string) ?? "text-embedding-3-small"
              }
              onChange={(e) => set("embedding_model", e.target.value)}
              placeholder="text-embedding-3-small"
              className="w-64 font-mono text-xs"
            />
          </Field>

          <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground/50 mt-4 mb-2">
            Storage
          </p>
          <Field
            label="DB Path"
            hint="Filesystem path for the embedded SurrealDB memory store."
          >
            <Input
              value={(val("db_path") as string) ?? "data/memory"}
              onChange={(e) => set("db_path", e.target.value)}
              placeholder="data/memory"
              className="w-64 font-mono text-xs"
            />
          </Field>

          <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground/50 mt-4 mb-2">
            MCP HTTP Endpoint
          </p>
          <div className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3 mb-1">
            <div>
              <p className="font-mono text-xs font-medium text-foreground">
                Enable MCP Endpoint
              </p>
              <p className="font-mono text-xs text-muted-foreground">
                Expose memory tools over HTTP as a Model Context Protocol
                server.
              </p>
            </div>
            <Toggle
              value={(val("mcp_http_enabled") as boolean) ?? false}
              onChange={(v) => set("mcp_http_enabled", v)}
            />
          </div>
          <Field label="MCP HTTP Path">
            <Input
              value={(val("mcp_http_path") as string) ?? "/mcp/memory"}
              onChange={(e) => set("mcp_http_path", e.target.value)}
              placeholder="/mcp/memory"
              className="w-48 font-mono text-xs"
            />
          </Field>
        </>
      )}
    </NamespacePanel>
  );
}

