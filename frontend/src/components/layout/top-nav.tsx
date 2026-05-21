import { Bot, Info, Menu, Settings2, X } from "lucide-react";
import { Link, useLocation } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { useUiStore } from "@/stores/ui-store";
import { cn } from "@/lib/utils";

const NAV_ITEMS = [
  { path: "/threads", label: "Chat", icon: Bot, exactMatch: false },
  { path: "/admin", label: "Admin", icon: Settings2, exactMatch: false },
  { path: "/about", label: "About", icon: Info, exactMatch: true },
] as const;

export function TopNav() {
  const location = useLocation();
  const { mobileSidebarOpen, toggleMobileSidebar } = useUiStore();

  const isActive = (path: string, exact: boolean) => {
    if (exact) return location.pathname === path;
    return location.pathname.startsWith(path);
  };

  const isChatRoute = location.pathname.startsWith("/threads");

  return (
    <header className="flex h-12 shrink-0 items-center gap-2 border-b border-border bg-card px-3">
      {/* Mobile sidebar toggle — only shown on chat view */}
      {isChatRoute && (
        <Button
          variant="ghost"
          size="icon"
          className="md:hidden h-8 w-8 shrink-0"
          onClick={toggleMobileSidebar}
          aria-label={mobileSidebarOpen ? "Close sidebar" : "Open sidebar"}
        >
          {mobileSidebarOpen ? <X size={18} /> : <Menu size={18} />}
        </Button>
      )}

      {/* Brand */}
      <div className="flex items-center gap-2 mr-4">
        <div className="flex size-7 items-center justify-center rounded-md bg-primary/20">
          <Bot size={14} className="text-primary" />
        </div>
        <span className="font-display text-sm font-semibold text-foreground hidden sm:block">Universal Agent Runtime</span>
        <span className="font-display text-sm font-semibold text-foreground sm:hidden">UAR</span>
      </div>

      {/* Nav links */}
      <nav className="flex items-center gap-1" role="navigation" aria-label="Main navigation">
        {NAV_ITEMS.map(({ path, label, icon: Icon, exactMatch }) => (
          <Button
            key={path}
            render={<Link to={path} />}
            variant="ghost"
            size="sm"
            className={cn(
              "h-8 gap-1.5 px-3 font-ui text-xs font-medium",
              isActive(path, exactMatch)
                ? "bg-accent text-foreground"
                : "text-muted-foreground hover:text-foreground",
            )}
            aria-current={isActive(path, exactMatch) ? "page" : undefined}
          >
            <Icon size={14} />
            {label}
          </Button>
        ))}
      </nav>
    </header>
  );
}
