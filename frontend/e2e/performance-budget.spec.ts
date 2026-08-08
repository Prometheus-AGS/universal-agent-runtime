import { expect, test } from "@playwright/test";

import {
  assertPerformanceBudget,
  PERFORMANCE_BUDGETS,
} from "../src/test/performance-budget";

test("cold IndexedDB thread list reaches its first hydrated browser-frame boundary within budget", async ({ context, page }, testInfo) => {
  expect((await context.storageState()).origins).toEqual([]);
  await page.route("**/api/uar/resolve-model", async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ ok: true, provider_id: "budget", model_id: "budget" }),
    });
  });

  await page.goto("/threads", { waitUntil: "domcontentloaded" });
  const threadList = page.getByLabel("Thread list");
  await expect(threadList).toBeVisible();
  await expect(threadList).toHaveAttribute("aria-busy", "false");
  await expect(page.getByText("No threads yet", { exact: true })).toBeVisible();
  const observedMs = await page.evaluate(() => {
    const mark = performance.getEntriesByName("uar-thread-list:first-paint-frame").at(-1);
    if (!mark) {
      throw new Error("Missing uar-thread-list:first-paint-frame performance mark");
    }
    return mark.startTime;
  });

  const indexedDatabases = await page.evaluate(async () => (
    await indexedDB.databases()
  ).map((database) => database.name ?? "<unnamed>"));
  const bootMarks = await page.evaluate(() => performance.getEntriesByType("mark").map((entry) => ({
    name: entry.name,
    startTime: entry.startTime,
  })));
  const pgliteResources = await page.evaluate(() => performance
    .getEntriesByType("resource")
    .filter((entry) => entry.name.includes("pglite") || entry.name.includes("initdb"))
    .map((entry) => ({
      name: entry.name,
      startTime: entry.startTime,
      duration: entry.duration,
      transferSize: "transferSize" in entry ? entry.transferSize : undefined,
    })));
  expect(indexedDatabases).toEqual(["/pglite/uar-threads"]);
  await testInfo.attach("coldThreadListFirstPaint", {
    body: JSON.stringify({
      name: "coldThreadListFirstPaint",
      observedMs,
      limitMs: PERFORMANCE_BUDGETS.coldThreadListFirstPaint,
      indexedDatabases,
      bootMarks,
      pgliteResources,
    }, null, 2),
    contentType: "application/json",
  });
  const result = assertPerformanceBudget("coldThreadListFirstPaint", observedMs);
  console.info("[performance-budget]", JSON.stringify(result));
});
