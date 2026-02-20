import { type FC, type ReactNode, useState } from "react";
import {
  Bot,
  BookOpen,
  Code2,
  Key,
  Layers,
  Server,
  SlidersHorizontal,
  Wrench,
  Zap,
} from "lucide-react";
import { cn } from "@/lib/utils";

export type AdminSection =
  | "providers"
  | "models"
  | "skills"
  | "agents"
  | "tools"
  | "auth"
  | "knowledge"
  | "compiler"
  | "settings";

const NAV_ITEMS: { id: AdminSection; label: string; icon: FC<{ size?: number; className?: string }> }[] = [
  { id: "providers", label: "Providers", icon: Server },
  { id: "models", label: "Models", icon: Layers },
  { id: "skills", label: "Skills", icon: Zap },
  { id: "agents", label: "Agents", icon: Bot },
  { id: "tools", label: "Tools", icon: Wrench },
  { id: "auth", label: "API Keys", icon: Key },
  { id: "knowledge", label: "Knowledge", icon: BookOpen },
  { id: "compiler", label: "Compiler", icon: Code2 },
  { id: "settings", label: "Settings", icon: SlidersHorizontal },
];

interface AdminShellProps {
  renderContent: (section: AdminSection) => ReactNode;
}

export function AdminShell({ renderContent }: AdminShellProps) {
  const [active, setActive] = useState<AdminSection>("providers");

  return (
    <div className="flex flex-1 overflow-hidden">
      {/* Inner sidebar */}
      <aside className="flex w-52 shrink-0 flex-col border-r border-border bg-card" role="navigation" aria-label="Admin navigation">
        <div className="border-b border-border px-4 py-3">
          <p className="font-mono text-[10px] font-medium uppercase tracking-widest text-muted-foreground">Administration</p>
        </div>
        <nav className="flex flex-col gap-0.5 overflow-y-auto p-2">
          {NAV_ITEMS.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActive(id)}
              className={cn(
                "flex items-center gap-2.5 rounded-md px-3 py-2 text-left text-sm font-medium transition-colors",
                active === id
                  ? "bg-accent text-foreground"
                  : "text-muted-foreground hover:bg-muted/50 hover:text-foreground",
              )}
              aria-current={active === id ? "page" : undefined}
            >
              <Icon size={15} className={active === id ? "text-primary" : "text-muted-foreground"} />
              {label}
            </button>
          ))}
        </nav>
      </aside>

      {/* Content area */}
      <main className="flex flex-1 flex-col overflow-hidden min-w-0">
        {renderContent(active)}
      </main>
    </div>
  );
}
