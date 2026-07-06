import { type FC, useState, useEffect, useCallback } from "react";
import { AlertTriangle, Loader2, Plus, X } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetDescription, SheetFooter } from "@/components/ui/sheet";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "@/components/ui/command";
import { ModelSelector } from "@/components/model-selector";
import { createAgent, updateAgentFull } from "@/services/agents-api";
import { fetchSkillsList } from "@/services/skills-api";
import { fetchToolsDiscovery } from "@/services/tools-api";
import { fetchKnowledgeBases } from "@/services/knowledge-api";
import type { UarAgent, UarSkill, UarTool, UarKnowledgeBase } from "@/types";

// ── Form state type ──────────────────────────────────────────────────────────

interface AgentFormState {
  // Identity
  title: string;
  description: string;
  status: string;
  default_model: string;
  fallback_model: string;

  // Prompt
  system_prompt: string;

  // Capabilities
  skills: string[];
  skill_selection_method: string;
  tools: string[];
  knowledge_bases: string[];
  kb_citation_required: boolean;

  // Memory
  memory_enabled: boolean;
  auto_capture: boolean;
  inject_context: boolean;
  memory_scope: string;

  // Governance
  tool_approval: string;
}

function defaultFormState(): AgentFormState {
  return {
    title: "",
    description: "",
    status: "draft",
    default_model: "",
    fallback_model: "",
    system_prompt: "",
    skills: [],
    skill_selection_method: "auto",
    tools: [],
    knowledge_bases: [],
    kb_citation_required: false,
    memory_enabled: true,
    auto_capture: false,
    inject_context: false,
    memory_scope: "agent",
    tool_approval: "auto",
  };
}

function parseModelString(modelStr: string): { provider: string; model: string } {
  const slash = modelStr.indexOf("/");
  if (slash > 0) {
    return { provider: modelStr.slice(0, slash), model: modelStr.slice(slash + 1) };
  }
  return { provider: "openai", model: modelStr || "gpt-4o" };
}

function formStateFromAgent(agent: UarAgent): AgentFormState {
  const raw = agent as unknown as Record<string, unknown>;
  const policy = (raw.policy as Record<string, unknown>) ?? {};
  const providerPolicy = (policy.provider as Record<string, unknown>) ?? {};
  const providerDefault = (providerPolicy.default as Record<string, unknown>) ?? {};
  const fallbacks = (providerPolicy.fallbacks as Array<Record<string, unknown>>) ?? [];
  const skillsPolicy = (policy.skills as Record<string, unknown>) ?? {};
  const toolsPolicy = (policy.tools as Record<string, unknown>) ?? {};
  const memory = (raw.memory as Record<string, unknown>) ?? {};
  const kb = (memory.kb as Record<string, unknown>) ?? {};
  const conversation = (memory.conversation as Record<string, unknown>) ?? {};
  const extensions = (raw.extensions as Record<string, unknown>) ?? {};

  return {
    title: agent.metadata?.title ?? agent.id,
    description: agent.metadata?.description ?? "",
    status: (raw.status as string) ?? (extensions.status as string) ?? "active",
    default_model:
      providerDefault.provider && providerDefault.model
        ? `${providerDefault.provider as string}/${providerDefault.model as string}`
        : "",
    fallback_model:
      fallbacks[0]?.provider && fallbacks[0]?.model
        ? `${fallbacks[0].provider as string}/${fallbacks[0].model as string}`
        : "",
    system_prompt: ((raw.prompt as Record<string, unknown>)?.system as string) ?? "",
    skills: (skillsPolicy.prefer as string[]) ?? [],
    skill_selection_method: (skillsPolicy.selection_method as string) ?? "auto",
    tools: (toolsPolicy.allow as string[]) ?? [],
    knowledge_bases: (kb.knowledge_bases as string[]) ?? [],
    kb_citation_required: (kb.citation_required as boolean) ?? false,
    memory_enabled: (conversation.enabled as boolean) ?? true,
    auto_capture: (extensions.memory_auto_capture as boolean) ?? false,
    inject_context: (extensions.memory_inject_context as boolean) ?? false,
    memory_scope: (extensions.memory_scope as string) ?? "agent",
    tool_approval: (extensions.tool_approval as string) ?? "auto",
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
      provider: {
        default: parseModelString(form.default_model),
        fallbacks: form.fallback_model
          ? [parseModelString(form.fallback_model)]
          : [],
      },
      tools: {
        allow: form.tools,
        deny: [],
        max_concurrent: 1,
      },
      skills: {
        prefer: form.skills,
        max_active: 3,
        selection_method: form.skill_selection_method,
      },
    },
    schemas: { inputs: null, outputs: null, state: null },
    prompt: {
      system: form.system_prompt,
      instructions: [],
    },
    memory: {
      conversation: { enabled: form.memory_enabled },
      kb: {
        enabled: form.knowledge_bases.length > 0,
        knowledge_bases: form.knowledge_bases,
        citation_required: form.kb_citation_required,
      },
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

  // Available items for chip selectors
  const [availableSkills, setAvailableSkills] = useState<UarSkill[]>([]);
  const [availableTools, setAvailableTools] = useState<UarTool[]>([]);
  const [availableKbs, setAvailableKbs] = useState<UarKnowledgeBase[]>([]);
  const [skillsLoading, setSkillsLoading] = useState(false);
  const [toolsLoading, setToolsLoading] = useState(false);
  const [kbsLoading, setKbsLoading] = useState(false);

  useEffect(() => {
    if (open) {
      setForm(agent ? formStateFromAgent(agent) : defaultFormState());
      setError(null);
    }
  }, [open, agent]);

  const update = <K extends keyof AgentFormState>(key: K, value: AgentFormState[K]) => {
    setForm((prev) => ({ ...prev, [key]: value }));
  };

  // ── Fetch helpers (called when popovers open) ─────────────────────────────

  const loadSkills = useCallback(() => {
    if (availableSkills.length > 0 || skillsLoading) return;
    setSkillsLoading(true);
    fetchSkillsList()
      .then(setAvailableSkills)
      .catch(() => {/* silent */})
      .finally(() => setSkillsLoading(false));
  }, [availableSkills.length, skillsLoading]);

  const loadTools = useCallback(() => {
    if (availableTools.length > 0 || toolsLoading) return;
    setToolsLoading(true);
    fetchToolsDiscovery()
      .then((res) => {
        const data = res.data ?? res;
        const all = [...(data.tools ?? []), ...(data.built_in_tools ?? [])];
        setAvailableTools(all);
      })
      .catch(() => {/* silent */})
      .finally(() => setToolsLoading(false));
  }, [availableTools.length, toolsLoading]);

  const loadKbs = useCallback(() => {
    if (availableKbs.length > 0 || kbsLoading) return;
    setKbsLoading(true);
    fetchKnowledgeBases()
      .then(setAvailableKbs)
      .catch(() => {/* silent */})
      .finally(() => setKbsLoading(false));
  }, [availableKbs.length, kbsLoading]);

  // ── Array helpers ─────────────────────────────────────────────────────────

  const addSkill = (id: string) => {
    if (!form.skills.includes(id)) update("skills", [...form.skills, id]);
  };
  const removeSkill = (id: string) => {
    update("skills", form.skills.filter((s) => s !== id));
  };
  const skillTitle = (id: string) => {
    const found = availableSkills.find((s) => s.skill_id === id);
    return found?.title ?? id;
  };

  const toolName = (t: UarTool) => t.namespaced_name ?? t.name;
  const addTool = (name: string) => {
    if (!form.tools.includes(name)) update("tools", [...form.tools, name]);
  };
  const removeTool = (name: string) => {
    update("tools", form.tools.filter((t) => t !== name));
  };

  const addKb = (id: string) => {
    if (!form.knowledge_bases.includes(id)) update("knowledge_bases", [...form.knowledge_bases, id]);
  };
  const removeKb = (id: string) => {
    update("knowledge_bases", form.knowledge_bases.filter((k) => k !== id));
  };
  const kbLabel = (id: string) => {
    const found = availableKbs.find((k) => k.id === id);
    return found ? found.name : id;
  };
  const kbDocCount = (id: string) => {
    const found = availableKbs.find((k) => k.id === id);
    return found?.document_count ?? 0;
  };

  // ── Group tools by namespace ──────────────────────────────────────────────

  const toolsByNamespace = (): Map<string, UarTool[]> => {
    const groups = new Map<string, UarTool[]>();
    for (const t of availableTools) {
      const name = toolName(t);
      const sep = name.indexOf("__");
      const ns = sep > 0 ? name.slice(0, sep) : (t.server ?? "built-in");
      const arr = groups.get(ns) ?? [];
      arr.push(t);
      groups.set(ns, arr);
    }
    return groups;
  };

  // ── Save ──────────────────────────────────────────────────────────────────

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
              <Select value={form.status} onValueChange={(v) => v !== null && update("status", v)}>
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
            <Separator />
            <div className="space-y-2">
              <Label className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Default Model</Label>
              <ModelSelector
                value={form.default_model}
                onChange={(v) => update("default_model", v)}
                placeholder="Select default model..."
              />
              {!form.default_model && (
                <div className="flex items-center gap-1.5 rounded-md bg-amber-500/10 px-2.5 py-1.5">
                  <AlertTriangle size={12} className="shrink-0 text-amber-500" />
                  <p className="font-mono text-[10px] text-amber-600 dark:text-amber-400">
                    No model set — agent will use the global default or fail if none is configured.
                  </p>
                </div>
              )}
            </div>
            <div className="space-y-2">
              <Label className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Fallback Model</Label>
              <ModelSelector
                value={form.fallback_model}
                onChange={(v) => update("fallback_model", v)}
                placeholder="Select fallback model (optional)..."
              />
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
            {/* Skills Section */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Skills</Label>
                <Popover onOpenChange={(o) => { if (o) loadSkills(); }}>
                  <PopoverTrigger render={<Button variant="outline" size="sm" className="h-7 gap-1 text-xs" />}>
                    <Plus size={12} /> Add
                  </PopoverTrigger>
                  <PopoverContent className="w-[300px] p-0" align="end">
                    <Command>
                      <CommandInput placeholder="Search skills..." className="font-mono text-xs" />
                      <CommandList>
                        {skillsLoading && (
                          <div className="py-3 text-center font-mono text-xs text-muted-foreground">Loading...</div>
                        )}
                        <CommandEmpty className="py-3 text-center font-mono text-xs text-muted-foreground">No skills found</CommandEmpty>
                        <CommandGroup>
                          {availableSkills
                            .filter((s) => !form.skills.includes(s.skill_id))
                            .map((s) => (
                              <CommandItem
                                key={s.skill_id}
                                value={`${s.title} ${s.skill_id}`}
                                onSelect={() => addSkill(s.skill_id)}
                                className="font-mono text-xs"
                              >
                                {s.title}
                              </CommandItem>
                            ))}
                        </CommandGroup>
                      </CommandList>
                    </Command>
                  </PopoverContent>
                </Popover>
              </div>
              <div className="flex flex-wrap gap-1.5">
                {form.skills.map((id) => (
                  <Badge key={id} variant="secondary" className="gap-1 font-mono text-xs">
                    {skillTitle(id)}
                    <button type="button" onClick={() => removeSkill(id)} className="ml-0.5 rounded-full hover:bg-muted">
                      <X size={10} />
                    </button>
                  </Badge>
                ))}
                {form.skills.length === 0 && (
                  <p className="font-mono text-xs text-muted-foreground">No skills selected</p>
                )}
              </div>
              <div className="space-y-1 pt-1">
                <Label className="font-mono text-[10px] uppercase tracking-widest text-muted-foreground">Selection Method</Label>
                <Select value={form.skill_selection_method} onValueChange={(v) => v !== null && update("skill_selection_method", v)}>
                  <SelectTrigger className="h-8 font-mono text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="auto" className="font-mono text-xs">Auto</SelectItem>
                    <SelectItem value="keyword" className="font-mono text-xs">Keyword</SelectItem>
                    <SelectItem value="embedding" className="font-mono text-xs">Embedding</SelectItem>
                    <SelectItem value="hybrid" className="font-mono text-xs">Hybrid</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <Separator />

            {/* Tools Section */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Tools (allow list)</Label>
                <Popover onOpenChange={(o) => { if (o) loadTools(); }}>
                  <PopoverTrigger render={<Button variant="outline" size="sm" className="h-7 gap-1 text-xs" />}>
                    <Plus size={12} /> Add
                  </PopoverTrigger>
                  <PopoverContent className="w-[300px] p-0" align="end">
                    <Command>
                      <CommandInput placeholder="Search tools..." className="font-mono text-xs" />
                      <CommandList>
                        {toolsLoading && (
                          <div className="py-3 text-center font-mono text-xs text-muted-foreground">Loading...</div>
                        )}
                        <CommandEmpty className="py-3 text-center font-mono text-xs text-muted-foreground">No tools found</CommandEmpty>
                        {[...toolsByNamespace().entries()].map(([ns, tools]) => (
                          <CommandGroup
                            key={ns}
                            heading={
                              <span className="font-mono text-[10px] font-medium uppercase tracking-widest text-muted-foreground">
                                {ns}
                              </span>
                            }
                          >
                            {tools
                              .filter((t) => !form.tools.includes(toolName(t)))
                              .map((t) => {
                                const name = toolName(t);
                                return (
                                  <CommandItem
                                    key={name}
                                    value={`${ns} ${name} ${t.description ?? ""}`}
                                    onSelect={() => addTool(name)}
                                    className="font-mono text-xs"
                                  >
                                    <span className="truncate">{name}</span>
                                  </CommandItem>
                                );
                              })}
                          </CommandGroup>
                        ))}
                      </CommandList>
                    </Command>
                  </PopoverContent>
                </Popover>
              </div>
              <div className="flex flex-wrap gap-1.5">
                {form.tools.map((name) => (
                  <Badge key={name} variant="secondary" className="gap-1 font-mono text-xs">
                    {name}
                    <button type="button" onClick={() => removeTool(name)} className="ml-0.5 rounded-full hover:bg-muted">
                      <X size={10} />
                    </button>
                  </Badge>
                ))}
                {form.tools.length === 0 && (
                  <p className="font-mono text-xs text-muted-foreground">No tools selected</p>
                )}
              </div>
            </div>

            <Separator />

            {/* Knowledge Bases Section */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label className="font-mono text-xs uppercase tracking-widest text-muted-foreground">Knowledge Bases</Label>
                <Popover onOpenChange={(o) => { if (o) loadKbs(); }}>
                  <PopoverTrigger render={<Button variant="outline" size="sm" className="h-7 gap-1 text-xs" />}>
                    <Plus size={12} /> Add
                  </PopoverTrigger>
                  <PopoverContent className="w-[300px] p-0" align="end">
                    <Command>
                      <CommandInput placeholder="Search knowledge bases..." className="font-mono text-xs" />
                      <CommandList>
                        {kbsLoading && (
                          <div className="py-3 text-center font-mono text-xs text-muted-foreground">Loading...</div>
                        )}
                        <CommandEmpty className="py-3 text-center font-mono text-xs text-muted-foreground">No knowledge bases found</CommandEmpty>
                        <CommandGroup>
                          {availableKbs
                            .filter((k) => !form.knowledge_bases.includes(k.id))
                            .map((k) => (
                              <CommandItem
                                key={k.id}
                                value={`${k.name} ${k.id}`}
                                onSelect={() => addKb(k.id)}
                                className="font-mono text-xs"
                              >
                                <span className="truncate">{k.name}</span>
                                {(k.document_count ?? 0) > 0 && (
                                  <span className="ml-auto shrink-0 rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
                                    {k.document_count} docs
                                  </span>
                                )}
                              </CommandItem>
                            ))}
                        </CommandGroup>
                      </CommandList>
                    </Command>
                  </PopoverContent>
                </Popover>
              </div>
              <div className="flex flex-wrap gap-1.5">
                {form.knowledge_bases.map((id) => (
                  <Badge key={id} variant="secondary" className="gap-1 font-mono text-xs">
                    {kbLabel(id)}
                    {kbDocCount(id) > 0 && (
                      <span className="rounded bg-muted-foreground/20 px-1 text-[10px]">{kbDocCount(id)}</span>
                    )}
                    <button type="button" onClick={() => removeKb(id)} className="ml-0.5 rounded-full hover:bg-muted">
                      <X size={10} />
                    </button>
                  </Badge>
                ))}
                {form.knowledge_bases.length === 0 && (
                  <p className="font-mono text-xs text-muted-foreground">No knowledge bases selected</p>
                )}
              </div>
              <div className="flex items-center justify-between pt-1">
                <div>
                  <p className="font-mono text-xs font-medium text-foreground">Citation Required</p>
                  <p className="mt-0.5 font-mono text-[10px] text-muted-foreground">Require citations from KB in responses.</p>
                </div>
                <Switch
                  checked={form.kb_citation_required}
                  onCheckedChange={(v) => update("kb_citation_required", v)}
                />
              </div>
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
              <Select value={form.memory_scope} onValueChange={(v) => v !== null && update("memory_scope", v)}>
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
              <Select value={form.tool_approval} onValueChange={(v) => v !== null && update("tool_approval", v)}>
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
