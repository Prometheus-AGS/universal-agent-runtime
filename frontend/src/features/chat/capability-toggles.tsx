import { type FC, useState, useCallback, useMemo } from "react";
import {
  Brain,
  Wrench,
  Zap,
  Globe,
  Database,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { cn } from "@/lib/utils";
import type { AgentConfig } from "@/features/chat/agent-selector";

// ─── Types ───────────────────────────────────────────────────────────────────

interface CapabilityTogglesProps {
  threadId: string | null;
  agentConfig?: AgentConfig | null;
  className?: string;
}

interface ListItem {
  id: string;
  label: string;
  enabled: boolean;
}

// ─── Toggle Button (simple on/off) ──────────────────────────────────────────

interface SimpleToggleProps {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onToggle: () => void;
}

const SimpleToggle: FC<SimpleToggleProps> = ({ icon, label, active, onToggle }) => (
  <Button
    type="button"
    variant="ghost"
    onClick={onToggle}
    aria-label={`${active ? "Disable" : "Enable"} ${label}`}
    className={cn(
      "h-7 gap-1.5 rounded-full px-2.5 text-xs font-medium transition-colors",
      active
        ? "bg-primary/15 text-primary hover:bg-primary/20"
        : "text-muted-foreground hover:bg-muted hover:text-foreground",
    )}
  >
    {icon}
    <span>{label}</span>
  </Button>
);

// ─── Toggle Button with Count + Popover ─────────────────────────────────────

interface ListToggleProps {
  icon: React.ReactNode;
  label: string;
  items: ListItem[];
  onToggleItem: (id: string) => void;
  onToggleAll: (enabled: boolean) => void;
}

const CountBadge: FC<{ count: number }> = ({ count }) => {
  if (count === 0) return null;
  return (
    <span className="inline-flex items-center justify-center rounded-full bg-primary/20 px-1.5 text-xs font-medium text-primary">
      {count}
    </span>
  );
};

const ListToggle: FC<ListToggleProps> = ({
  icon,
  label,
  items,
  onToggleItem,
  onToggleAll,
}) => {
  const activeCount = items.filter((i) => i.enabled).length;
  const allEnabled = items.length > 0 && activeCount === items.length;

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          aria-label={`Toggle ${label}`}
          className={cn(
            "h-7 gap-1.5 rounded-full px-2.5 text-xs font-medium transition-colors",
            activeCount > 0
              ? "bg-primary/15 text-primary hover:bg-primary/20"
              : "text-muted-foreground hover:bg-muted hover:text-foreground",
          )}
        >
          {icon}
          <span>{label}</span>
          <CountBadge count={activeCount} />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        side="top"
        align="start"
        className="w-64 p-0"
        onOpenAutoFocus={(e) => e.preventDefault()}
      >
        {/* Header with toggle-all */}
        <div className="flex items-center justify-between border-b border-border px-3 py-2">
          <span className="font-mono text-[11px] uppercase tracking-wider text-muted-foreground">
            {label}
          </span>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-6 px-2 text-xs"
            onClick={() => onToggleAll(!allEnabled)}
          >
            {allEnabled ? "Disable all" : "Enable all"}
          </Button>
        </div>

        {/* Item list */}
        <div className="max-h-48 overflow-y-auto p-1">
          {items.length === 0 ? (
            <p className="px-3 py-4 text-center text-xs text-muted-foreground">
              No {label.toLowerCase()} available
            </p>
          ) : (
            items.map((item) => (
              <label
                key={item.id}
                className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-muted/50"
              >
                <Checkbox
                  checked={item.enabled}
                  onCheckedChange={() => onToggleItem(item.id)}
                />
                <span className="truncate">{item.label}</span>
              </label>
            ))
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
};

// ─── Main Component ─────────────────────────────────────────────────────────

type CapabilityKind = "knowledgeBases" | "tools" | "skills";

type DisabledByKind = Record<CapabilityKind, string[]>;

const EMPTY_DISABLED: DisabledByKind = {
  knowledgeBases: [],
  tools: [],
  skills: [],
};

function toListItems(ids: string[] | undefined, disabled: string[]): ListItem[] {
  const disabledSet = new Set(disabled);
  return (ids ?? []).map((id) => ({
    id,
    label: id,
    enabled: !disabledSet.has(id),
  }));
}

export const CapabilityToggles: FC<CapabilityTogglesProps> = ({
  agentConfig,
  className,
}) => {
  const configSignature = useMemo(
    () => JSON.stringify({
      knowledgeBases: agentConfig?.knowledge_bases ?? [],
      skills: agentConfig?.skills ?? [],
      tools: agentConfig?.tools ?? [],
    }),
    [agentConfig],
  );
  const [disabledBySignature, setDisabledBySignature] = useState<Record<string, DisabledByKind>>({});
  const [webSearchEnabled, setWebSearchEnabled] = useState(false);
  const [memoryEnabled, setMemoryEnabled] = useState(true);

  const disabled = disabledBySignature[configSignature] ?? EMPTY_DISABLED;
  const knowledgeBases = useMemo(
    () => toListItems(agentConfig?.knowledge_bases, disabled.knowledgeBases),
    [agentConfig?.knowledge_bases, disabled.knowledgeBases],
  );
  const tools = useMemo(
    () => toListItems(agentConfig?.tools, disabled.tools),
    [agentConfig?.tools, disabled.tools],
  );
  const skills = useMemo(
    () => toListItems(agentConfig?.skills, disabled.skills),
    [agentConfig?.skills, disabled.skills],
  );

  // ── List toggle helpers ──
  const updateDisabled = useCallback(
    (kind: CapabilityKind, updater: (current: string[]) => string[]) => {
      setDisabledBySignature((prev) => {
        const current = prev[configSignature] ?? EMPTY_DISABLED;
        return {
          ...prev,
          [configSignature]: {
            ...current,
            [kind]: updater(current[kind]),
          },
        };
      });
    },
    [configSignature],
  );

  const toggleItem = useCallback(
    (kind: CapabilityKind, id: string) => {
      updateDisabled(kind, (current) =>
        current.includes(id)
          ? current.filter((itemId) => itemId !== id)
          : [...current, id],
      );
    },
    [updateDisabled],
  );

  const toggleAll = useCallback(
    (kind: CapabilityKind, items: ListItem[], enabled: boolean) => {
      updateDisabled(kind, () => enabled ? [] : items.map((item) => item.id));
    },
    [updateDisabled],
  );

  return (
    <div
      className={cn(
        "flex gap-1.5 overflow-x-auto scrollbar-none",
        className,
      )}
    >
      <ListToggle
        icon={<Brain className="size-3.5" />}
        label="Knowledge"
        items={knowledgeBases}
        onToggleItem={(id) => toggleItem("knowledgeBases", id)}
        onToggleAll={(enabled) => toggleAll("knowledgeBases", knowledgeBases, enabled)}
      />

      <ListToggle
        icon={<Wrench className="size-3.5" />}
        label="Tools"
        items={tools}
        onToggleItem={(id) => toggleItem("tools", id)}
        onToggleAll={(enabled) => toggleAll("tools", tools, enabled)}
      />

      <ListToggle
        icon={<Zap className="size-3.5" />}
        label="Skills"
        items={skills}
        onToggleItem={(id) => toggleItem("skills", id)}
        onToggleAll={(enabled) => toggleAll("skills", skills, enabled)}
      />

      <SimpleToggle
        icon={<Globe className="size-3.5" />}
        label="Web Search"
        active={webSearchEnabled}
        onToggle={() => setWebSearchEnabled((v) => !v)}
      />

      <SimpleToggle
        icon={<Database className="size-3.5" />}
        label="Memory"
        active={memoryEnabled}
        onToggle={() => setMemoryEnabled((v) => !v)}
      />
    </div>
  );
};
