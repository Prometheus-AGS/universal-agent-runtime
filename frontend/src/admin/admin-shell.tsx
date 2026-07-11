import { type FC, useEffect, useMemo, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import {
  Activity,
  BadgeCheck,
  Bot,
  BookOpen,
  Brain,
  Code2,
  Compass,
  DollarSign,
  FileClock,
  Key,
  KeyRound,
  Layers,
  LayoutGrid,
  Menu,
  RadioTower,
  Search,
  Server,
  SlidersHorizontal,
  SquareActivity,
  Wrench,
  X,
  Zap,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useRuntimeConsoleFeeds } from "@/entities/runtime-console-feeds";
import { Button } from "@/components/ui/button";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandShortcut,
} from "@/components/ui/command";

export type AdminSection =
  | "runtime"
  | "runs"
  | "approvals"
  | "protocols"
  | "providers"
  | "credentials"
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
  | "mcp-health"
  | "cost";

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
    label: "Runtime",
    items: [
      { id: "runtime", label: "Cockpit", subtitle: "Live runtime state", icon: SquareActivity },
      { id: "runs", label: "Runs", subtitle: "Steps, tools, artifacts", icon: FileClock },
      { id: "approvals", label: "Approvals", subtitle: "Human interrupts", icon: BadgeCheck },
      { id: "protocols", label: "Protocols", subtitle: "AG-UI, A2UI, REST", icon: RadioTower },
    ],
  },
  {
    label: "AI & Models",
    items: [
      { id: "providers", label: "Providers", subtitle: "LLM provider config", icon: Server },
      { id: "credentials", label: "Credentials", subtitle: "Your provider API keys", icon: KeyRound },
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
      { id: "cost", label: "Cost", subtitle: "Spend & budget alerts", icon: DollarSign },
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
  return match ? match.id : "runtime";
}

interface AdminShellProps {
  renderContent: (section: AdminSection) => React.ReactNode;
}

export function AdminShell({ renderContent }: AdminShellProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const active = sectionFromPath(location.pathname);

  // Poll the REST-backed Runtime Console feeds (Provider Health, A2UI Surfaces,
  // Model Routing) into the runtime entity graph while the console is open.
  useRuntimeConsoleFeeds();
  const [mobileNavOpen, setMobileNavOpen] = useState(false);
  const [commandOpen, setCommandOpen] = useState(false);

  const activeItem = useMemo(
    () => ALL_ITEMS.find((item) => item.id === active),
    [active],
  );

  const goTo = (id: AdminSection) => {
    navigate(`/admin/${id}`);
    setMobileNavOpen(false);
    setCommandOpen(false);
  };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setCommandOpen((open) => !open);
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, []);

  const navContent = (
    <>
      <div className="border-b border-border px-4 py-3">
        <div className="flex items-center gap-2">
          <div className="flex size-7 items-center justify-center rounded-md border border-primary/25 bg-primary/10">
            <Compass size={14} className="text-primary" />
          </div>
          <div>
            <p className="font-display text-sm font-semibold text-foreground">
              Runtime Console
            </p>
            <p className="font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
              UAR operations
            </p>
          </div>
        </div>
      </div>
      <nav className="flex flex-col gap-3 overflow-y-auto p-2">
        {NAV_GROUPS.map((group) => (
          <div key={group.label}>
            <p className="mb-1 px-3 font-mono text-xs font-medium text-muted-foreground/70">
              {group.label}
            </p>
            <div className="flex flex-col gap-0.5">
              {group.items.map(({ id, label, subtitle, icon: Icon }) => (
                <button
                  key={id}
                  onClick={() => goTo(id)}
                  data-testid={`admin-nav-${id}`}
                  className={cn(
                    "grid cursor-pointer grid-cols-[16px_1fr] items-center gap-x-2.5 rounded-md px-3 py-2 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
                    active === id
                      ? "bg-accent text-foreground"
                      : "text-muted-foreground hover:bg-muted/50 hover:text-foreground",
                  )}
                  aria-current={active === id ? "page" : undefined}
                >
                  <Icon
                    size={15}
                    className={cn(
                      "mt-0.5",
                      active === id ? "text-primary" : "text-muted-foreground"
                    )}
                  />
                  <span>
                    <span className="block text-sm font-medium leading-5">{label}</span>
                    <span className="block truncate text-[11px] leading-4 text-muted-foreground">
                      {subtitle}
                    </span>
                  </span>
                </button>
              ))}
            </div>
          </div>
        ))}
      </nav>
    </>
  );

  return (
    <div className="flex flex-1 overflow-hidden" data-testid="admin-shell">
      <CommandDialog open={commandOpen} onOpenChange={setCommandOpen}>
        <CommandInput placeholder="Search runtime, providers, protocols..." />
        <CommandList>
          <CommandEmpty>No runtime surface found.</CommandEmpty>
          {NAV_GROUPS.map((group) => (
            <CommandGroup key={group.label} heading={group.label}>
              {group.items.map(({ id, label, subtitle, icon: Icon }) => (
                <CommandItem
                  key={id}
                  value={`${label} ${subtitle}`}
                  onSelect={() => goTo(id)}
                >
                  <Icon size={15} />
                  <div className="min-w-0">
                    <p className="text-sm">{label}</p>
                    <p className="truncate text-xs text-muted-foreground">{subtitle}</p>
                  </div>
                  <CommandShortcut>{id}</CommandShortcut>
                </CommandItem>
              ))}
            </CommandGroup>
          ))}
        </CommandList>
      </CommandDialog>

      {/* Mobile overlay */}
      {mobileNavOpen && (
        <div
          data-testid="admin-mobile-nav-overlay"
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
        data-testid="admin-navigation"
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
        <div
          className="flex h-12 shrink-0 items-center justify-between border-b border-border bg-card/80 px-4 backdrop-blur"
          data-testid="admin-topbar"
        >
          <div className="flex min-w-0 items-center gap-3">
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 shrink-0 md:hidden"
              onClick={() => setMobileNavOpen(!mobileNavOpen)}
              aria-label={mobileNavOpen ? "Close admin navigation" : "Open admin navigation"}
              data-testid="admin-mobile-nav-trigger"
            >
              {mobileNavOpen ? <X size={16} /> : <Menu size={16} />}
            </Button>
            <div className="min-w-0">
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <span>Console</span>
                <span>/</span>
                <span className="truncate text-foreground">{activeItem?.label ?? active}</span>
              </div>
              <p className="truncate text-xs text-muted-foreground">
                {activeItem?.subtitle ?? "Runtime operations"}
              </p>
            </div>
          </div>
          <Button
            variant="outline"
            size="sm"
            className="hidden h-8 gap-2 sm:inline-flex"
            onClick={() => setCommandOpen(true)}
            data-testid="admin-command-trigger"
          >
            <Search size={14} />
            Command
            <kbd className="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
              ⌘K
            </kbd>
          </Button>
        </div>
        <div className="flex min-h-0 flex-1 overflow-hidden">
          {renderContent(active)}
        </div>
      </main>
    </div>
  );
}
