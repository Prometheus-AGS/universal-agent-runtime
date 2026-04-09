import { type FC } from "react";
import { Brain, Search, Wrench } from "lucide-react";
import { cn } from "@/lib/utils";
import type { AgentStatusType } from "@/stores/agent-status-store";

interface AgentStatusIndicatorProps {
  status: { type: AgentStatusType; toolName?: string };
}

const statusConfig = {
  thinking: {
    Icon: Brain,
    label: "Thinking...",
    iconClass: "animate-pulse text-primary",
  },
  executing: {
    Icon: Wrench,
    label: "Executing",
    iconClass: "animate-spin text-warning",
  },
  searching: {
    Icon: Search,
    label: "Searching...",
    iconClass: "animate-pulse text-blue-400",
  },
  idle: {
    Icon: Brain,
    label: "",
    iconClass: "",
  },
} as const;

export const AgentStatusIndicator: FC<AgentStatusIndicatorProps> = ({ status }) => {
  const isVisible = status.type !== "idle";
  const config = statusConfig[status.type];
  const { Icon } = config;

  const label =
    status.type === "executing" && status.toolName
      ? `Executing ${status.toolName}...`
      : config.label;

  return (
    <div
      className={cn(
        "flex items-center gap-2 transition-opacity duration-300 ease-in-out",
        isVisible ? "opacity-100" : "pointer-events-none opacity-0",
      )}
      aria-live="polite"
      aria-label={label}
    >
      {isVisible && (
        <>
          <Icon size={14} className={cn("shrink-0", config.iconClass)} />
          <span className="font-mono text-[11px] text-muted-foreground">
            {label}
          </span>
        </>
      )}
    </div>
  );
};
