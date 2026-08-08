import { Dialog } from "@base-ui/react/dialog";
import { useEffect } from "react";
import { X } from "lucide-react";
import { Link } from "react-router";

import { useUiState } from "@/hooks/use-ui-state";

import { CONFIGURE_DESTINATIONS } from "./nav-destinations";

export function MobileSheetHost() {
  const { shellSheet, setShellSheet } = useUiState();
  const open = shellSheet === "configure";

  useEffect(() => {
    if (!open || typeof window.matchMedia !== "function") return;

    const desktopQuery = window.matchMedia("(min-width: 901px)");
    const closeOnDesktop = () => {
      if (desktopQuery.matches) setShellSheet(null);
    };

    closeOnDesktop();
    desktopQuery.addEventListener("change", closeOnDesktop);
    return () => desktopQuery.removeEventListener("change", closeOnDesktop);
  }, [open, setShellSheet]);

  return (
    <Dialog.Root
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) setShellSheet(null);
      }}
    >
      <Dialog.Portal>
        <Dialog.Backdrop className="fixed inset-0 z-40 bg-background/80 transition-opacity duration-200 data-ending-style:opacity-0 data-starting-style:opacity-0 min-[901px]:hidden" />
        <Dialog.Viewport className="fixed inset-0 z-50 flex items-end p-2 pb-[calc(.5rem+env(safe-area-inset-bottom))] min-[901px]:hidden">
          <Dialog.Popup className="max-h-[min(80dvh,38rem)] w-full overflow-y-auto rounded-2xl bg-card p-3 text-foreground transition-[translate,opacity] duration-200 data-ending-style:translate-y-5 data-ending-style:opacity-0 data-starting-style:translate-y-5 data-starting-style:opacity-0">
            <div className="flex items-start justify-between gap-3 px-2 pb-3 pt-1">
              <div>
                <Dialog.Title className="font-display text-lg font-semibold">Configure</Dialog.Title>
                <Dialog.Description className="mt-1 text-sm text-fg-sub">
                  Runtime connections, capabilities, and settings.
                </Dialog.Description>
              </div>
              <Dialog.Close className="flex size-11 shrink-0 items-center justify-center rounded-xl text-fg-sub transition-colors duration-200 hover:bg-card-hov hover:text-foreground focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember">
                <X aria-hidden="true" className="size-5" />
                <span className="sr-only">Close Configure</span>
              </Dialog.Close>
            </div>

            <nav aria-label="Configure destinations" className="grid gap-1">
              {CONFIGURE_DESTINATIONS.map((destination) => {
                const Icon = destination.icon;
                return (
                  <Link
                    key={destination.id}
                    to={destination.path}
                    onClick={() => setShellSheet(null)}
                    className="flex min-h-14 items-center gap-3 rounded-xl px-3 text-left transition-colors duration-200 hover:bg-card-hov focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember"
                  >
                    <span className="flex size-10 shrink-0 items-center justify-center rounded-xl bg-surface text-ember">
                      <Icon aria-hidden="true" className="size-[18px]" />
                    </span>
                    <span className="min-w-0">
                      <span className="block text-sm font-semibold text-foreground">{destination.label}</span>
                      <span className="block truncate text-xs text-fg-sub">{destination.description}</span>
                    </span>
                  </Link>
                );
              })}
            </nav>
          </Dialog.Popup>
        </Dialog.Viewport>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
