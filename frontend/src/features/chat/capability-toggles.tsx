import { type FC, useState, useCallback } from "react";
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

// ─── Types ───────────────────────────────────────────────────────────────────

interface CapabilityTogglesProps {
  threadId: string | null;
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

// Placeholder data for V1 — will be wired to real APIs later
const INITIAL_KBS: ListItem[] = [];
const INITIAL_TOOLS: ListItem[] = [];
const INITIAL_SKILLS: ListItem[] = [];

export const CapabilityToggles: FC<CapabilityTogglesProps> = ({
  threadId: _threadId,
  className,
}) => {
  // ── Local state (V1 — will wire to agent session later) ──
  const [knowledgeBases, setKnowledgeBases] = useState<ListItem[]>(INITIAL_KBS);
  const [tools, setTools] = useState<ListItem[]>(INITIAL_TOOLS);
  const [skills, setSkills] = useState<ListItem[]>(INITIAL_SKILLS);
  const [webSearchEnabled, setWebSearchEnabled] = useState(false);
  const [memoryEnabled, setMemoryEnabled] = useState(true);

  // ── List toggle helpers ──
  const toggleItem = useCallback(
    (setter: React.Dispatch<React.SetStateAction<ListItem[]>>, id: string) => {
      setter((prev) =>
        prev.map((item) =>
          item.id === id ? { ...item, enabled: !item.enabled } : item,
        ),
      );
    },
    [],
  );

  const toggleAll = useCallback(
    (setter: React.Dispatch<React.SetStateAction<ListItem[]>>, enabled: boolean) => {
      setter((prev) => prev.map((item) => ({ ...item, enabled })));
    },
    [],
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
        onToggleItem={(id) => toggleItem(setKnowledgeBases, id)}
        onToggleAll={(enabled) => toggleAll(setKnowledgeBases, enabled)}
      />

      <ListToggle
        icon={<Wrench className="size-3.5" />}
        label="Tools"
        items={tools}
        onToggleItem={(id) => toggleItem(setTools, id)}
        onToggleAll={(enabled) => toggleAll(setTools, enabled)}
      />

      <ListToggle
        icon={<Zap className="size-3.5" />}
        label="Skills"
        items={skills}
        onToggleItem={(id) => toggleItem(setSkills, id)}
        onToggleAll={(enabled) => toggleAll(setSkills, enabled)}
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
