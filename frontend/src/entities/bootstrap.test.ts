import { afterEach, describe, expect, test, vi } from "vitest";

import type {
  GraphPersistenceAdapter,
  LocalFirstGraphRuntime,
  PGlitePersistenceClient,
} from "@/platform/entities";
import {
  bootstrapDurableEntityGraph,
  initializeDurableEntityGraph,
  teardownEntityGraph,
} from "@/entities/bootstrap";

describe("durable entity graph bootstrap", () => {
  afterEach(() => {
    teardownEntityGraph();
  });

  test("hydrates local-first state before realtime sync", async () => {
    const order: string[] = [];
    const db = {} as PGlitePersistenceClient;
    const storage = {} as GraphPersistenceAdapter;
    let resolveReady: (() => void) | undefined;
    const runtime = {
      ready: new Promise<void>((resolve) => {
        resolveReady = resolve;
      }),
      dispose: vi.fn(),
      persistNow: vi.fn(),
      hydrate: vi.fn(),
      getStatus: vi.fn(),
    } as unknown as LocalFirstGraphRuntime;

    const result = await initializeDurableEntityGraph(db, {
      createStorage: vi.fn(async () => {
        order.push("storage");
        return storage;
      }),
      startLocalFirst: vi.fn(() => {
        order.push("runtime");
        order.push("hydrate");
        resolveReady?.();
        return runtime;
      }),
      startSync: vi.fn(async () => {
        order.push("sync");
        return vi.fn();
      }),
    });

    expect(order).toEqual(["storage", "runtime", "hydrate", "sync"]);
    expect(result.runtime).toBe(runtime);
  });

  test("uses package snapshot storage without an application outbox", async () => {
    const source = await import("@/platform/pglite/migrations");
    expect(source.RUN_EVENT_MIGRATION_SQL).not.toMatch(/CREATE TABLE IF NOT EXISTS outbox/i);
  });

  test("disposes a failed runtime and permits a later bootstrap retry", async () => {
    const db = {} as PGlitePersistenceClient;
    const storage = {} as GraphPersistenceAdapter;
    const failedRuntime = {
      ready: Promise.reject(new Error("hydrate failed")),
      dispose: vi.fn(),
    } as unknown as LocalFirstGraphRuntime;
    const healthyRuntime = {
      ready: Promise.resolve(),
      dispose: vi.fn(),
    } as unknown as LocalFirstGraphRuntime;
    const startLocalFirst = vi
      .fn<(storage: GraphPersistenceAdapter) => LocalFirstGraphRuntime>()
      .mockReturnValueOnce(failedRuntime)
      .mockReturnValueOnce(healthyRuntime);
    const dependencies = {
      createStorage: vi.fn(async () => storage),
      startLocalFirst,
      startSync: vi.fn(async () => vi.fn()),
    };

    await expect(bootstrapDurableEntityGraph(db, dependencies)).rejects.toThrow(
      "hydrate failed",
    );
    await expect(bootstrapDurableEntityGraph(db, dependencies)).resolves.toBeUndefined();

    expect(failedRuntime.dispose).toHaveBeenCalledOnce();
    expect(startLocalFirst).toHaveBeenCalledTimes(2);
    expect(dependencies.startSync).toHaveBeenCalledOnce();
  });
});
