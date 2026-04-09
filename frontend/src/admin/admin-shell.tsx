import { type FC, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import {
  Activity,
  Bot,
  BookOpen,
  Brain,
  Code2,
  Key,
  Layers,
  LayoutGrid,
  Menu,
  Server,
  SlidersHorizontal,
  Wrench,
  X,
  Zap,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

export type AdminSection =
  | "providers"
  | "models"
  | "skills"
  | "agents"
  | "tools"
  | "auth"
  | "knowledge"
  | "memory"
  | "compiler"
  | "settings"
  | "a2ui-testing"
  | "mcp-health";

interface NavGroup {
  label: string;
  items: {
    id: AdminSection;
    label: string;
    subtitle: string;
    icon: FC<{ size?: number; className?: string }>;
  }[];
}

const NAV_GROUPS: NavGroup[] = [
  {
    label: "AI & Models",
    items: [
      { id: "providers", label: "Providers", subtitle: "LLM provider config", icon: Server },
      { id: "models", label: "Models", subtitle: "Browse model catalog", icon: Layers },
      { id: "knowledge", label: "Knowledge", subtitle: "Document stores & RAG", icon: BookOpen },
      { id: "memory", label: "Memory", subtitle: "Agent memory items", icon: Brain },
    ],
  },
  {
    label: "Agents & Tools",
    items: [
      { id: "agents", label: "Agents", subtitle: "Configure agents", icon: Bot },
      { id: "skills", label: "Skills", subtitle: "WASM skill registry", icon: Zap },
      { id: "tools", label: "Tools", subtitle: "Discovered MCP tools", icon: Wrench },
    ],
  },
  {
    label: "Infrastructure",
    items: [
      { id: "auth", label: "API Keys", subtitle: "Authentication keys", icon: Key },
      { id: "settings", label: "Settings", subtitle: "Global configuration", icon: SlidersHorizontal },
      { id: "mcp-health", label: "MCP Health", subtitle: "Server connections", icon: Activity },
    ],
  },
  {
    label: "Development",
    items: [
      { id: "compiler", label: "Compiler", subtitle: "WASM compilation", icon: Code2 },
      { id: "a2ui-testing", label: "A2UI Testing", subtitle: "Artifact schemas", icon: LayoutGrid },
    ],
  },
];

// Flat lookup for resolving section from URL
const ALL_ITEMS = NAV_GROUPS.flatMap((g) => g.items);

function sectionFromPath(pathname: string): AdminSection {
  const segment = pathname.replace(/^\/admin\/?/, "").split("/")[0];
  const match = ALL_ITEMS.find((item) => item.id === segment);
  return match ? match.id : "providers";
}

interface AdminShellProps {
  renderContent: (section: AdminSection) => React.ReactNode;
}

export function AdminShell({ renderContent }: AdminShellProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const active = sectionFromPath(location.pathname);
  const [mobileNavOpen, setMobileNavOpen] = useState(false);

  const goTo = (id: AdminSection) => {
    navigate(`/admin/${id}`);
    setMobileNavOpen(false);
  };

  const navContent = (
    <>
      <div className="border-b border-border px-4 py-3">
        <p className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">
          Administration
        </p>
      </div>
      <nav className="flex flex-col gap-3 overflow-y-auto p-2">
        {NAV_GROUPS.map((group) => (
          <div key={group.label}>
            <p className="mb-1 px-3 font-mono text-xs font-medium text-muted-foreground/70">
              {group.label}
            </p>
            <div className="flex flex-col gap-0.5">
              {group.items.map(({ id, label, icon: Icon }) => (
                <button
                  key={id}
                  onClick={() => goTo(id)}
                  className={cn(
                    "flex cursor-pointer items-center gap-2.5 rounded-md px-3 py-2 text-left text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
                    active === id
                      ? "bg-accent text-foreground"
                      : "text-muted-foreground hover:bg-muted/50 hover:text-foreground",
                  )}
                  aria-current={active === id ? "page" : undefined}
                >
                  <Icon
                    size={15}
                    className={
                      active === id ? "text-primary" : "text-muted-foreground"
                    }
                  />
                  {label}
                </button>
              ))}
            </div>
          </div>
        ))}
      </nav>
    </>
  );

  return (
    <div className="flex flex-1 overflow-hidden">
      {/* Mobile nav toggle */}
      <div className="flex items-center border-b border-border bg-card px-3 py-2 md:hidden absolute top-0 left-0 z-30">
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={() => setMobileNavOpen(!mobileNavOpen)}
          aria-label={mobileNavOpen ? "Close admin navigation" : "Open admin navigation"}
        >
          {mobileNavOpen ? <X size={16} /> : <Menu size={16} />}
        </Button>
      </div>

      {/* Mobile overlay */}
      {mobileNavOpen && (
        <div
          className="fixed inset-0 z-40 bg-background/60 md:hidden"
          onClick={() => setMobileNavOpen(false)}
          onKeyDown={(e) => {
            if (e.key === "Escape") setMobileNavOpen(false);
          }}
          role="button"
          tabIndex={0}
          aria-label="Close navigation"
        />
      )}

      {/* Sidebar — desktop: always visible, mobile: slide-over */}
      <aside
        className={cn(
          "flex shrink-0 flex-col border-r border-border bg-card transition-transform duration-200",
          // Desktop
          "hidden md:flex md:w-52",
          // Mobile overlay
          mobileNavOpen &&
            "fixed inset-y-0 left-0 z-50 flex w-64 shadow-xl md:relative md:w-52 md:shadow-none",
        )}
        role="navigation"
        aria-label="Admin navigation"
      >
        {navContent}
      </aside>

      {/* Content area */}
      <main className="flex flex-1 flex-col overflow-hidden min-w-0">
        {renderContent(active)}
      </main>
    </div>
  );
}
