import { Autocomplete } from "@base-ui/react/autocomplete";
import { Dialog } from "@base-ui/react/dialog";
import { CornerDownLeft, X } from "lucide-react";
import { useNavigate } from "react-router";

import { useUiState } from "@/hooks/use-ui-state";

import {
  NAV_DESTINATIONS,
  type NavigationDestination,
} from "./nav-destinations";

export function CommandPalette() {
  const navigate = useNavigate();
  const { commandPaletteOpen, setCommandPaletteOpen } = useUiState();

  const navigateTo = (destination: NavigationDestination) => {
    navigate(destination.path);
    setCommandPaletteOpen(false);
  };

  return (
    <Dialog.Root
      open={commandPaletteOpen}
      onOpenChange={(open) => {
        setCommandPaletteOpen(open);
      }}
    >
      <Dialog.Portal>
        <Dialog.Backdrop className="fixed inset-0 z-50 bg-background/80 transition-opacity duration-200 data-ending-style:opacity-0 data-starting-style:opacity-0" />
        <Dialog.Viewport className="fixed inset-0 z-[60] flex items-start justify-center overflow-hidden px-3 pt-[min(12vh,6rem)]">
          <Dialog.Popup
            aria-label="Command palette"
            className="flex max-h-[min(70dvh,34rem)] w-full max-w-xl flex-col overflow-hidden rounded-2xl bg-card text-foreground transition-[translate,opacity] duration-200 data-ending-style:-translate-y-5 data-ending-style:opacity-0 data-starting-style:-translate-y-5 data-starting-style:opacity-0"
          >
            <Dialog.Title className="sr-only">Command palette</Dialog.Title>
            <Autocomplete.Root
              open
              inline
              items={NAV_DESTINATIONS}
              itemToStringValue={(destination) => `${destination.label} ${destination.description}`}
              autoHighlight="always"
              keepHighlight
            >
              <div className="flex items-center gap-2 bg-surface p-2">
                <Autocomplete.Input
                  autoFocus
                  aria-label="Search commands"
                  placeholder="Search routes and commands…"
                  className="h-11 min-w-0 flex-1 rounded-xl bg-card px-3 text-base text-foreground outline-none placeholder:text-fg-faint focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember"
                />
                <Dialog.Close className="flex size-11 shrink-0 items-center justify-center rounded-xl text-fg-sub transition-colors duration-200 hover:bg-card-hov hover:text-foreground focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember">
                  <X aria-hidden="true" className="size-5" />
                  <span className="sr-only">Close command palette</span>
                </Dialog.Close>
              </div>

              <Autocomplete.Empty>
                <div className="flex min-h-28 items-center px-4 py-6 text-sm text-fg-sub">
                  No commands found.
                </div>
              </Autocomplete.Empty>

              <Autocomplete.List className="min-h-0 overflow-y-auto p-2 data-empty:p-0">
                {(destination: NavigationDestination) => {
                  const Icon = destination.icon;
                  return (
                    <Autocomplete.Item
                      key={destination.id}
                      value={destination}
                      // Base UI fires Item.onClick for pointer activation and Enter on a
                      // highlighted item while the input or list has focus.
                      onClick={() => navigateTo(destination)}
                      className="group flex min-h-14 cursor-default items-center gap-3 rounded-xl px-3 text-left outline-none select-none data-highlighted:bg-card-hov data-highlighted:text-foreground"
                    >
                      <span className="flex size-10 shrink-0 items-center justify-center rounded-xl bg-surface text-ember group-data-highlighted:bg-ember-soft">
                        <Icon aria-hidden="true" className="size-[18px]" />
                      </span>
                      <span className="min-w-0 flex-1">
                        <span className="block text-sm font-semibold">{destination.label}</span>
                        <span className="block truncate text-xs text-fg-sub">{destination.description}</span>
                      </span>
                      <span className="font-mono text-[10px] uppercase tracking-wider text-fg-faint">
                        {destination.group}
                      </span>
                    </Autocomplete.Item>
                  );
                }}
              </Autocomplete.List>

              <div className="flex items-center justify-between bg-surface px-4 py-2.5 text-xs text-fg-sub">
                <span>Navigate with arrow keys</span>
                <span className="flex items-center gap-1.5 font-mono">
                  <CornerDownLeft aria-hidden="true" className="size-3.5" />
                  Enter
                </span>
              </div>
            </Autocomplete.Root>
          </Dialog.Popup>
        </Dialog.Viewport>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
