import { type ReactNode, useCallback, useEffect, useRef } from "react";
import { useLocation } from "react-router";

import { useHealthz } from "@/hooks/use-healthz";
import { useUiState } from "@/hooks/use-ui-state";

import { BreadcrumbHeader } from "./breadcrumb-header";
import { CommandPalette } from "./command-palette";
import { MobileSheetHost } from "./mobile-sheet-host";
import { MobileTabBar } from "./mobile-tab-bar";
import { NavRail } from "./nav-rail";

function isEditableTarget(target: EventTarget | null): boolean {
  return target instanceof HTMLElement && Boolean(
    target.closest("input, textarea, select, [contenteditable]:not([contenteditable='false'])"),
  );
}

function hasOpenModalDialog(): boolean {
  return Array.from(document.querySelectorAll('[role="dialog"][aria-modal="true"]')).some(
    (dialog) => !dialog.hasAttribute("hidden") && dialog.getAttribute("aria-hidden") !== "true",
  );
}

export function AppShell({ children }: { children: ReactNode }) {
  const { pathname } = useLocation();
  const { health } = useHealthz();
  const { commandPaletteOpen, setCommandPaletteOpen, closeShellOverlays } = useUiState();
  const commandPaletteReturnFocusRef = useRef<HTMLElement | null>(null);
  const commandPaletteShouldRestoreFocusRef = useRef(false);

  const openCommandPalette = useCallback(() => {
    if (document.activeElement instanceof HTMLElement) {
      commandPaletteReturnFocusRef.current = document.activeElement;
    }
    commandPaletteShouldRestoreFocusRef.current = true;
    setCommandPaletteOpen(true);
  }, [setCommandPaletteOpen]);

  useEffect(() => {
    closeShellOverlays();
  }, [pathname, closeShellOverlays]);

  useEffect(() => {
    if (commandPaletteOpen || !commandPaletteShouldRestoreFocusRef.current) return;

    let secondFrame = 0;
    const firstFrame = requestAnimationFrame(() => {
      secondFrame = requestAnimationFrame(() => {
        const priorTrigger = commandPaletteReturnFocusRef.current;
        const currentTrigger = document.querySelector<HTMLElement>(
          '[aria-label="Open command palette"]',
        );
        const focusTarget = priorTrigger?.isConnected ? priorTrigger : currentTrigger;
        focusTarget?.focus();
        commandPaletteShouldRestoreFocusRef.current = false;
      });
    });

    return () => {
      cancelAnimationFrame(firstFrame);
      cancelAnimationFrame(secondFrame);
    };
  }, [commandPaletteOpen, pathname]);

  useEffect(() => {
    const handleCommandPaletteShortcut = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        if (commandPaletteOpen) {
          event.preventDefault();
          setCommandPaletteOpen(false);
          return;
        }
        if (isEditableTarget(event.target) || hasOpenModalDialog()) return;
        event.preventDefault();
        openCommandPalette();
      }
    };

    document.addEventListener("keydown", handleCommandPaletteShortcut);
    return () => document.removeEventListener("keydown", handleCommandPaletteShortcut);
  }, [commandPaletteOpen, openCommandPalette, setCommandPaletteOpen]);

  return (
    // The shared theme's reduced-motion media query forces these shell transitions to 1ms.
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <a
        href="#shell-main-content"
        className="fixed left-3 top-3 z-[80] -translate-y-20 rounded-xl bg-ember px-4 py-3 font-semibold text-ember-fg transition-transform duration-200 focus:translate-y-0 focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-ember"
      >
        Skip to content
      </a>

      <div className="flex min-h-0 w-full flex-1">
        <NavRail pathname={pathname} health={health} />
        <div className="flex min-h-0 min-w-0 flex-1 flex-col bg-background">
          <BreadcrumbHeader pathname={pathname} health={health} onOpenCommandPalette={openCommandPalette} />
          <main
            id="shell-main-content"
            tabIndex={-1}
            className="flex min-h-0 min-w-0 flex-1 overflow-hidden bg-background outline-none"
          >
            {children}
          </main>
        </div>
      </div>

      <MobileTabBar pathname={pathname} />
      <MobileSheetHost />
      <CommandPalette />
    </div>
  );
}
