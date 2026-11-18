import { useCallback, useRef } from "react";

/**
 * Focus return for sheets: remember the control that opened the sheet and give focus back when it closes.
 * If that control is gone (the sheet created something that replaced it) or disabled, focus the preferred
 * element, then the detail heading, so focus never falls back to the document body.
 */
export function useSheetFocusReturn(fallbackId = "teams-detail-heading") {
  const opener = useRef<HTMLElement | null>(null);

  const remember = useCallback(() => {
    opener.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  }, []);

  const restore = useCallback(
    (preferredId?: string) => {
      const target = opener.current;
      // Two frames: the sheet unmounts and the list it changed re-renders before focus lands.
      requestAnimationFrame(() =>
        requestAnimationFrame(() => {
          const preferred = preferredId ? document.getElementById(preferredId) : null;
          const usable = target && target.isConnected && !target.hasAttribute("disabled") ? target : null;
          (preferred ?? usable ?? document.getElementById(fallbackId))?.focus();
        }),
      );
    },
    [fallbackId],
  );

  return { remember, restore };
}

/**
 * One command id per distinct request. A retry of an unchanged request reuses it (the runtime then treats
 * it as the same command); changing the inputs renews it. Created only in event handlers.
 */
export function useCommandId() {
  const current = useRef<string | null>(null);
  const get = useCallback(() => (current.current ??= crypto.randomUUID()), []);
  const renew = useCallback(() => {
    current.current = null;
  }, []);
  return { get, renew };
}
