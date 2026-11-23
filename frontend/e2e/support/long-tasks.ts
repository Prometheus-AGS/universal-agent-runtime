import type { Page } from "@playwright/test";

export interface LongTask {
  startTime: number;
  duration: number;
}

export interface LongTaskReading {
  /** False when the browser cannot report long tasks; a reading is then meaningless. */
  supported: boolean;
  tasks: LongTask[];
  maxDurationMs: number;
}

interface LongTaskState {
  supported: boolean;
  tasks: LongTask[];
  observer: PerformanceObserver | null;
}

const INIT_SCRIPT = `(() => {
  const state = { supported: false, tasks: [], observer: null };
  window.__uarLongTasks = state;
  try {
    state.supported = (PerformanceObserver.supportedEntryTypes || []).includes("longtask");
    if (state.supported) {
      state.observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          state.tasks.push({ startTime: entry.startTime, duration: entry.duration });
        }
      });
      state.observer.observe({ type: "longtask" });
    }
  } catch (_error) {
    state.supported = false;
  }
})();`;

/** Starts recording `longtask` entries in every document the page loads. Call before `goto`. */
export async function installLongTaskObserver(page: Page): Promise<void> {
  await page.addInitScript(INIT_SCRIPT);
}

/** Discards everything recorded so far, including entries the observer has queued but not delivered. */
export async function resetLongTasks(page: Page): Promise<void> {
  await page.evaluate(() => {
    const state = (window as unknown as { __uarLongTasks?: LongTaskState }).__uarLongTasks;
    if (!state) return;
    state.observer?.takeRecords();
    state.tasks = [];
  });
}

/** Returns every long task recorded since the last reset, draining entries still queued. */
export async function readLongTasks(page: Page): Promise<LongTaskReading> {
  return page.evaluate(() => {
    const state = (window as unknown as { __uarLongTasks?: LongTaskState }).__uarLongTasks;
    if (!state) return { supported: false, tasks: [], maxDurationMs: 0 };
    for (const entry of state.observer?.takeRecords() ?? []) {
      state.tasks.push({ startTime: entry.startTime, duration: entry.duration });
    }
    const tasks = state.tasks.map((task) => ({ ...task }));
    return {
      supported: state.supported,
      tasks,
      maxDurationMs: tasks.reduce((max, task) => Math.max(max, task.duration), 0),
    };
  });
}
