import { useState, useEffect, useCallback } from "react";
import { ChevronDownIcon, CheckIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { fetchAgentsList } from "@/services/agents-api";
import { fetchConfiguredProviders } from "@/services/providers-api";
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
  const [agents, setAgents] = useState<AgentWithType[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(true);
  const [modelLabel, setModelLabel] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    fetchAgentsList()
      .then((list) => {
        if (!cancelled) setAgents(list as AgentWithType[]);
      })
      .catch(() => {
        /* swallow — show empty list */
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => { cancelled = true; };
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
      if (next) {
        void applyAgentConfig(next);
        const agent = agents.find((a) => a.id === next);
        if (agent) {
          const config = extractAgentConfig(agent);
          onAgentConfigChange?.(config);
          setModelLabel(config.model ?? "Using default model");
        }
      } else {
        onAgentConfigChange?.(null);
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
    [selectedId, applyAgentConfig, agents, onAgentConfigChange],
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
