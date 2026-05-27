import { useState, useEffect, useCallback, useMemo } from "react";
import { ChevronDownIcon, CheckIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { fetchConfiguredProviders } from "@/services/providers-api";
import { loadAgentsIntoGraph } from "@/entities/fetchers/agents";
import { useAgents } from "@/entities/hooks/use-agents";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import type { UarAgent } from "@/types";

/** Extracted agent configuration passed downstream to toggles & config panel. */
export interface AgentConfig {
  model?: string;
  skills: string[];
  tools: string[];
  knowledge_bases: string[];
  tool_approval?: string;
}

/** Extract a normalized AgentConfig from a full UarAgent. */
export function extractAgentConfig(agent: UarAgent): AgentConfig {
  const provider = agent.policy?.provider?.default;
  const model = provider?.provider && provider?.model
    ? `${provider.provider}/${provider.model}`
    : provider?.model ?? undefined;

  const skills = agent.policy?.skills?.prefer ?? [];

  // Gather tools from policy.tools.allow and tools.bundles
  const allowedTools = agent.policy?.tools?.allow ?? [];
  const bundleTools = (agent.tools?.bundles ?? []).flatMap((b) => b.tools ?? []);
  const tools = [...new Set([...allowedTools, ...bundleTools])];

  const knowledge_bases = agent.memory?.kb?.knowledge_bases ?? [];

  return { model, skills, tools, knowledge_bases };
}

interface AgentSelectorProps {
  threadId: string | null;
  onAgentConfigChange?: (config: AgentConfig | null) => void;
  className?: string;
}

type AgentWithType = UarAgent & { _type: "runtime" | "federated" };

export function AgentSelector({ threadId, onAgentConfigChange, className }: AgentSelectorProps) {
  // Agent list now comes from the entity graph — same source as the Admin
  // page — so SSE mutations (rename, delete, enable-flag flips) propagate
  // into the chat sidebar without a reload.
  const agentsView = useAgents();
  const agents = agentsView.items as unknown as AgentWithType[];
  const loading = agents.length === 0;

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [open, setOpen] = useState(false);
  const [modelLabel, setModelLabel] = useState<string | null>(null);

  // Hydrate the graph on mount (idempotent — the Admin page also calls this).
  useEffect(() => {
    void loadAgentsIntoGraph();
  }, []);

  // Fetch default provider/model to display in header
  useEffect(() => {
    let cancelled = false;
    fetchConfiguredProviders()
      .then((data) => {
        if (cancelled) return;
        const defaultId = data.default_id;
        if (defaultId) {
          const provider = (data.providers ?? []).find((p) => p.id === defaultId);
          if (provider?.models && provider.models.length > 0) {
            setModelLabel(`${defaultId}/${provider.models[0].id}`);
          } else {
            setModelLabel(`${defaultId}`);
          }
        } else if ((data.providers ?? []).length > 0) {
          setModelLabel("Using default model");
        }
      })
      .catch(() => {
        /* swallow */
      });
    return () => { cancelled = true; };
  }, []);

  // Render-derived AgentConfig: re-derives whenever the selected agent's
  // underlying record changes in the graph (e.g. via SSE from an admin edit
  // in another tab). Pushes the new config through the existing prop callback
  // so downstream consumers (capability toggles, session config panel,
  // chat hot path via useAgentConfig) always see fresh values.
  const currentAgent = useMemo(
    () => (selectedId ? agents.find((a) => a.id === selectedId) ?? null : null),
    [selectedId, agents],
  );
  const derivedConfig = useMemo(
    () => (currentAgent ? extractAgentConfig(currentAgent) : null),
    [currentAgent],
  );
  useEffect(() => {
    onAgentConfigChange?.(derivedConfig);
  }, [derivedConfig, onAgentConfigChange]);

  const applyAgentConfig = useCallback(
    async (agentId: string) => {
      if (!threadId) return;
      try {
        await fetch(`/api/uar/sessions/${encodeURIComponent(threadId)}/agent-config`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ agent_id: agentId }),
        });
      } catch {
        /* best-effort */
      }
    },
    [threadId],
  );

  const handleSelect = useCallback(
    (agentId: string) => {
      const next = agentId === selectedId ? null : agentId;
      setSelectedId(next);
      setOpen(false);
      // AgentConfig propagation now flows automatically via the
      // derivedConfig useEffect above; here we only handle side effects:
      // session-side apply + model-label refresh.
      if (next) {
        void applyAgentConfig(next);
        const agent = agents.find((a) => a.id === next);
        if (agent) {
          const config = extractAgentConfig(agent);
          setModelLabel(config.model ?? "Using default model");
        }
      } else {
        // Reset model label to system default
        fetchConfiguredProviders()
          .then((data) => {
            const defaultId = data.default_id;
            if (defaultId) {
              const provider = (data.providers ?? []).find((p) => p.id === defaultId);
              if (provider?.models && provider.models.length > 0) {
                setModelLabel(`${defaultId}/${provider.models[0].id}`);
              } else {
                setModelLabel(`${defaultId}`);
              }
            } else if ((data.providers ?? []).length > 0) {
              setModelLabel("Using default model");
            }
          })
          .catch(() => { /* swallow */ });
      }
    },
    [selectedId, applyAgentConfig, agents],
  );

  const selectedAgent = agents.find((a) => a.id === selectedId);
  const displayName = selectedAgent?.metadata?.title ?? selectedAgent?.id ?? "Default Assistant";

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          type="button"
          className={cn(
            "inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-left transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
            className,
          )}
          aria-label="Select agent"
        >
          <div className="min-w-0 flex-1">
            <span className="block truncate max-w-[200px] font-mono text-xs text-muted-foreground">{displayName}</span>
            {modelLabel && (
              <span className="block truncate max-w-[200px] font-mono text-[11px] text-muted-foreground/70">{modelLabel}</span>
            )}
          </div>
          <ChevronDownIcon size={12} className={cn("shrink-0 text-muted-foreground transition-transform", open && "rotate-180")} />
        </button>
      </PopoverTrigger>
      <PopoverContent className="w-[300px] p-0" align="start">
        <Command>
          <CommandInput placeholder="Search agents..." />
          <CommandList>
            <CommandEmpty>
              {loading ? "Loading agents..." : "No agents found."}
            </CommandEmpty>
            {agents.length > 0 && (
              <CommandGroup>
                {agents.map((agent) => {
                  const title = agent.metadata?.title ?? agent.id;
                  const desc = agent.metadata?.description;
                  return (
                    <CommandItem
                      key={agent.id}
                      value={`${title} ${desc ?? ""}`}
                      onSelect={() => handleSelect(agent.id)}
                      className="flex items-start gap-2"
                    >
                      <CheckIcon
                        size={14}
                        className={cn(
                          "mt-0.5 shrink-0",
                          selectedId === agent.id ? "opacity-100" : "opacity-0",
                        )}
                      />
                      <div className="min-w-0 flex-1">
                        <div className="font-mono text-xs font-medium truncate">{title}</div>
                        {desc && (
                          <div className="text-[11px] text-muted-foreground truncate">{desc}</div>
                        )}
                      </div>
                      {agent._type === "federated" && (
                        <span className="ml-auto shrink-0 rounded bg-muted px-1 py-0.5 text-[10px] text-muted-foreground">
                          federated
                        </span>
                      )}
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
