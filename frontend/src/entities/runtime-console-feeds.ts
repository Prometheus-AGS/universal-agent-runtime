import { useEffect } from "react";

import { useRuntimeConsoleStore } from "@/stores/runtime-console-store";

/** Poll normalized Runtime Console snapshots while the owning shell is mounted. */
export function useRuntimeConsoleFeeds(): void {
  const start = useRuntimeConsoleStore((state) => state.start);
  const stop = useRuntimeConsoleStore((state) => state.stop);

  useEffect(() => {
    start();
    return stop;
  }, [start, stop]);
}
