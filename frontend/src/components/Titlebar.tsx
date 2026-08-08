// Branded custom titlebar (uar-ui-shell-navigation). tauri.conf.json sets
// decorations:false, so the OS draws no chrome — this component owns drag,
// minimize/maximize/close, and the UAR lockup. Control placement follows
// platform convention: left (traffic-light order) on macOS, right on
// Windows/Linux.
//
// Renders ONLY inside Tauri — in a plain browser this returns null and no
// Tauri API is touched (every @tauri-apps/api call throws synchronously at
// module scope without the __TAURI_INTERNALS__ bridge).
//
// Adapted from desktop/src/shared/components/Titlebar.tsx. Deviation: UAR does
// not register tauri-plugin-os on the Rust side, so macOS detection uses a
// userAgent check (gated behind isTauri) instead of platform().
import { useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow, type Window as TauriWindow } from "@tauri-apps/api/window";
import { UarWordmark } from "@/shared/ui/uar-logo";

const inTauri = isTauri();
const appWindow: TauriWindow | null = inTauri ? getCurrentWindow() : null;

function WindowControlsMac() {
  return (
    <div className="flex items-center gap-2 pl-2">
      <button
        aria-label="Close"
        onClick={() => appWindow?.close()}
        className="h-3 w-3 rounded-full bg-[#FF5F57] hover:brightness-90 active:brightness-75"
      />
      <button
        aria-label="Minimize"
        onClick={() => appWindow?.minimize()}
        className="h-3 w-3 rounded-full bg-[#FEBC2E] hover:brightness-90 active:brightness-75"
      />
      <button
        aria-label="Maximize"
        onClick={() => appWindow?.toggleMaximize()}
        className="h-3 w-3 rounded-full bg-[#28C840] hover:brightness-90 active:brightness-75"
      />
    </div>
  );
}

function WindowControlsWindows() {
  return (
    <div className="flex items-stretch">
      <button
        aria-label="Minimize"
        onClick={() => appWindow?.minimize()}
        className="flex h-8 w-11 items-center justify-center text-fg-sub hover:bg-card-hov"
      >
        &#xE921;
      </button>
      <button
        aria-label="Maximize"
        onClick={() => appWindow?.toggleMaximize()}
        className="flex h-8 w-11 items-center justify-center text-fg-sub hover:bg-card-hov"
      >
        &#xE922;
      </button>
      <button
        aria-label="Close"
        onClick={() => appWindow?.close()}
        className="flex h-8 w-11 items-center justify-center text-fg-sub hover:bg-[#E81123] hover:text-white"
      >
        &#xE8BB;
      </button>
    </div>
  );
}

export function Titlebar() {
  const [isMac] = useState(() => {
    if (!inTauri) return false;
    try {
      return /Mac|iPhone|iPad/.test(window.navigator.userAgent);
    } catch {
      return false;
    }
  });

  if (!inTauri) return null;

  // data-tauri-drag-region marks the drag surface, but Tauri's own webview-side
  // listener only fires reliably on a direct mousedown against that element —
  // explicitly calling startDragging() makes the whole bar draggable, including
  // the flex spacers around the lockup.
  const startDrag = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    void appWindow?.startDragging();
  };

  return (
    <header
      data-tauri-drag-region
      onMouseDown={startDrag}
      className="flex h-9 shrink-0 select-none items-center bg-chrome"
    >
      {isMac && <WindowControlsMac />}
      <div className="flex flex-1 items-center justify-center text-foreground">
        <UarWordmark className="h-7 w-40" />
      </div>
      {!isMac && <WindowControlsWindows />}
    </header>
  );
}
