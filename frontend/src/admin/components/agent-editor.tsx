import { type FC, useState, useEffect } from "react";
import { Loader2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetDescription, SheetFooter } from "@/components/ui/sheet";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import { createAgent, updateAgentFull } from "@/services/agents-api";
import type { UarAgent } from "@/types";

// ── Form state type ──────────────────────────────────────────────────────────

interface AgentFormState {
  title: string;
  description: string;
  status: string;
  system_prompt: string;
  skills: string;
  tools: string;
  memory_enabled: boolean;
  auto_capture: boolean;
  inject_context: boolean;
  memory_scope: string;
  tool_approval: string;
}

function defaultFormState(): AgentFormState {
  return {
    title: "",
    description: "",
    status: "draft",
    system_prompt: "",
    skills: "",
    tools: "",
    memory_enabled: true,
    auto_capture: false,
    inject_context: false,
    memory_scope: "agent",
    tool_approval: "auto",
  };
}

function formStateFromAgent(agent: UarAgent): AgentFormState {
  const raw = agent as unknown as Record<string, unknown>;
  return {
    title: agent.metadata?.title ?? agent.id,
    description: agent.metadata?.description ?? "",
    status: (raw.status as string) ?? "active",
    system_prompt: (raw.system_prompt as string) ?? ((raw.prompt as Record<string, unknown>)?.system as string) ?? "",
    skills: agent.skills?.map((s) => s.skill_id ?? s.title).join(", ") ?? "",
    tools: (raw.tools as string) ?? "",
    memory_enabled: (raw.memory_enabled as boolean) ?? true,
    auto_capture: (raw.memory_auto_capture as boolean) ?? false,
    inject_context: (raw.memory_inject_context as boolean) ?? false,
    memory_scope: (raw.memory_scope as string) ?? "agent",
    tool_approval: (raw.tool_approval as string) ?? "auto",
  };
}

function buildArtifactPayload(form: AgentFormState, existingId?: string) {
  const id = existingId ?? (form.title.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "new-agent");
  return {
    version: "1.0",
    kind: "agent",
    id,
    metadata: {
      title: form.title,
      description: form.description,
      tags: [],
    },
    runtime: {
      entry: "default",
      protocols: {},
    },
    policy: {
      provider: { default: { provider: "openai", model: "gpt-4o" }, fallbacks: [] },
      tools: {
        allow: form.tools ? form.tools.split(",").map((s) => s.trim()).filter(Boolean) : [],
        deny: [],
        max_concurrent: 1,
      },
      skills: {
        prefer: form.skills ? form.skills.split(",").map((s) => s.trim()).filter(Boolean) : [],
        max_active: 3,
      },
    },
    schemas: { inputs: null, outputs: null, state: null },
    prompt: {
      system: form.system_prompt,
      instructions: [],
    },
    memory: {
      conversation: { enabled: form.memory_enabled },
      kb: { enabled: false, knowledge_bases: [], citation_required: false },
    },
    tools: { bundles: [] },
    ui: {
      forms: { enabled: false },
      artifacts: { enabled: false, preferred_types: [] },
    },
    extensions: {
      status: form.status,
      tool_approval: form.tool_approval,
      memory_auto_capture: form.auto_capture,
      memory_inject_context: form.inject_context,
      memory_scope: form.memory_scope,
    },
  };
}

// ── Scope options ────────────────────────────────────────────────────────────

const SCOPE_OPTIONS = [
  { value: "agent", label: "Agent-scoped" },
  { value: "user", label: "User-scoped" },
  { value: "global", label: "Global" },
  { value: "session", label: "Session-scoped" },
];

// ── Component ────────────────────────────────────────────────────────────────

interface AgentEditorProps {
  agent?: UarAgent | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: () => void;
}

export const AgentEditor: FC<AgentEditorProps> = ({ agent, open, onOpenChange, onSave }) => {
  const isCreate = !agent;
  const [form, setForm] = useState<AgentFormState>(defaultFormState);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setForm(agent ? formStateFromAgent(agent) : defaultFormState());
      setError(null);
    }
  }, [open, agent]);

  const update = <K extends keyof AgentFormState>(key: K, value: AgentFormState[K]) => {
    setForm((prev) => ({ ...prev, [key]: value }));
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      const payload = buildArtifactPayload(form, agent?.id);
      const res = isCreate
        ? await createAgent(payload)
        : await updateAgentFull(agent!.id, payload);
      if (!res.ok) {
        const text = await res.text();
        throw new Error(text || `${res.status}`);
      }
      onSave();
      onOpenChange(false);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="flex w-full flex-col sm:max-w-lg">
        <SheetHeader>
          <SheetTitle className="font-display">{isCreate ? "New Agent" : "Edit Agent"}</SheetTitle>
          <SheetDescription className="font-mono text-xs">
            {isCreate ? "Create a new agent with default settings." : `Editing ${agent?.metadata?.title ?? agent?.id}`}
          </SheetDescription>
        </SheetHeader>

        <Tabs defaultValue="identity" className="mt-4 flex flex-1 flex-col overflow-hidden">
          <TabsList className="w-full shrink-0">
            <TabsTrigger value="identity" className="flex-1 font-mono text-xs">Identity</TabsTrigger>
            <TabsTrigger value="prompt" className="flex-1 font-mono text-xs">Prompt</TabsTrigger>
            <TabsTrigger value="capabilities" className="flex-1 font-mono text-xs">Capabilities</TabsTrigger>
            <TabsTrigger value="memory" className="flex-1 font-mono text-xs">Memory</TabsTrigger>
            <TabsTrigger value="governance" className="flex-1 font-mono text-xs">Governance</TabsTrigger>
          </TabsList>

          {/* ── Identity ─────────────────────────────────────────────── */}
          <TabsContent value="identity" className="flex-1 space-y-4 overflow-y-auto pr-1">
            <div className="space-y-2">
              <Label htmlFor="agent-name" className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Name</Label>
              <Input
                id="agent-name"
                value={form.title}
                onChange={(e) => update("title", e.target.value)}
                placeholder="My Agent"
                className="font-display"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="agent-desc" className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Description</Label>
              <Textarea
                id="agent-desc"
                value={form.description}
                onChange={(e) => update("description", e.target.value)}
                placeholder="What does this agent do?"
                rows={3}
              />
            </div>
            <div className="space-y-2">
              <Label className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Status</Label>
              <Select value={form.status} onValueChange={(v) => update("status", v)}>
                <SelectTrigger className="font-mono text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="active" className="font-mono text-xs">Active</SelectItem>
                  <SelectItem value="draft" className="font-mono text-xs">Draft</SelectItem>
                  <SelectItem value="disabled" className="font-mono text-xs">Disabled</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </TabsContent>

          {/* ── Prompt ───────────────────────────────────────────────── */}
          <TabsContent value="prompt" className="flex flex-1 flex-col overflow-hidden pr-1">
            <div className="flex flex-1 flex-col space-y-2">
              <Label htmlFor="system-prompt" className="font-mono text-xs uppercase tracking-widest text-muted-foreground">System Prompt</Label>
              <Textarea
                id="system-prompt"
                value={form.system_prompt}
                onChange={(e) => update("system_prompt", e.target.value)}
                placeholder="You are a helpful assistant..."
                className="flex-1 resize-none font-mono text-xs"
              />
            </div>
          </TabsContent>

          {/* ── Capabilities ─────────────────────────────────────────── */}
          <TabsContent value="capabilities" className="flex-1 space-y-4 overflow-y-auto pr-1">
            <div className="space-y-2">
              <Label htmlFor="agent-skills" className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Skills</Label>
              <p className="font-mono text-xs text-muted-foreground">Comma-separated skill identifiers</p>
              <Input
                id="agent-skills"
                value={form.skills}
                onChange={(e) => update("skills", e.target.value)}
                placeholder="search, summarize, translate"
                className="font-mono text-xs"
              />
            </div>
            <Separator />
            <div className="space-y-2">
              <Label htmlFor="agent-tools" className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Tools (allow list)</Label>
              <p className="font-mono text-xs text-muted-foreground">Comma-separated tool names</p>
              <Input
                id="agent-tools"
                value={form.tools}
                onChange={(e) => update("tools", e.target.value)}
                placeholder="tavily::search, time::now"
                className="font-mono text-xs"
              />
            </div>
          </TabsContent>

          {/* ── Memory ───────────────────────────────────────────────── */}
          <TabsContent value="memory" className="flex-1 space-y-4 overflow-y-auto pr-1">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-mono text-xs font-medium text-foreground">Memory Enabled</p>
                <p className="mt-0.5 font-mono text-xs text-muted-foreground">Enable conversation memory for this agent.</p>
              </div>
              <Switch checked={form.memory_enabled} onCheckedChange={(v) => update("memory_enabled", v)} />
            </div>
            <Separator />
            <div className="flex items-center justify-between">
              <div>
                <p className="font-mono text-xs font-medium text-foreground">Auto-Capture</p>
                <p className="mt-0.5 font-mono text-xs text-muted-foreground">Extract memories after each turn.</p>
              </div>
              <Switch checked={form.auto_capture} onCheckedChange={(v) => update("auto_capture", v)} />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="font-mono text-xs font-medium text-foreground">Context Injection</p>
                <p className="mt-0.5 font-mono text-xs text-muted-foreground">Inject memories as system prompt prefix.</p>
              </div>
              <Switch checked={form.inject_context} onCheckedChange={(v) => update("inject_context", v)} />
            </div>
            <Separator />
            <div className="space-y-2">
              <Label className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Default Scope</Label>
              <Select value={form.memory_scope} onValueChange={(v) => update("memory_scope", v)}>
                <SelectTrigger className="font-mono text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {SCOPE_OPTIONS.map((o) => (
                    <SelectItem key={o.value} value={o.value} className="font-mono text-xs">
                      {o.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </TabsContent>

          {/* ── Governance ───────────────────────────────────────────── */}
          <TabsContent value="governance" className="flex-1 space-y-4 overflow-y-auto pr-1">
            <div className="space-y-2">
              <Label className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Tool Approval</Label>
              <p className="font-mono text-xs text-muted-foreground">How tool calls are approved before execution.</p>
              <Select value={form.tool_approval} onValueChange={(v) => update("tool_approval", v)}>
                <SelectTrigger className="font-mono text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="auto" className="font-mono text-xs">Auto (no approval needed)</SelectItem>
                  <SelectItem value="ask" className="font-mono text-xs">Ask (user confirms each call)</SelectItem>
                  <SelectItem value="deny" className="font-mono text-xs">Deny (block all tool calls)</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </TabsContent>
        </Tabs>

        {error && (
          <p className="mt-2 rounded-md bg-destructive/10 px-3 py-2 font-mono text-xs text-destructive">{error}</p>
        )}

        <SheetFooter className="mt-4 shrink-0">
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={saving}>
            Cancel
          </Button>
          <Button onClick={() => void handleSave()} disabled={saving || !form.title.trim()} className="gap-1.5">
            {saving && <Loader2 size={14} className="animate-spin" />}
            {isCreate ? "Create Agent" : "Save Changes"}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
};
