/**
 * SCN-IDN-001: catalog model identity survives shared bare model ids.
 *
 * Actor: an operator browsing /admin/models. Outcome: every provider's model is
 * shown once, under its own provider, even when 1,123 bare model ids are listed by
 * several providers. Production entry: the UI with the collision config; the real
 * runtime, the real GET /api/models and the browser are all real.
 *
 * Oracle (frozen, never edited): catalog-identity.json and collision-config.yaml,
 * through the copies and digests in ./support/frozen-fixtures.ts. Comparator: exact
 * count and per-row field equality.
 *
 * Declared limit: the graph is not read directly; rows are read through the rendered
 * page. On the pre-change tree the REST path already keys by provider/model, so the
 * main test is a regression guard there (its non-triviality comes from the bare-id
 * mutant, which collapses 8,173 rows to about 3,830 and must fail the count).
 *
 * No credentials, tokens or Authorization headers are used anywhere in this file.
 */
import crypto from "node:crypto";
import { expect, test } from "@playwright/test";
import {
  loadCatalogSnapshot,
  loadIdentityOracle,
  type FrozenModelRow,
} from "./support/frozen-fixtures";
import {
  expectCatalogRow,
  filterCatalog,
  readCatalogCounts,
  waitForCatalogTotal,
} from "./support/models-page";

interface ApiModelEntry {
  limit?: { context?: number | null };
  cost?: { input?: number; output?: number } | null;
  modalities?: { input?: string[] };
  tool_call?: boolean;
  reasoning?: boolean;
}

type ApiModels = Record<string, { models?: Record<string, ApiModelEntry> }>;

function countApiPairs(api: ApiModels): number {
  return Object.values(api).reduce(
    (sum, provider) => sum + Object.keys(provider.models ?? {}).length,
    0,
  );
}

function expectApiEntryMatches(api: ApiModels, row: FrozenModelRow): void {
  const entry = api[row.provider_id]?.models?.[row.model_id];
  expect(entry, `GET /api/models lists ${row.id}`).toBeDefined();
  if (!entry) return;
  expect(entry.limit?.context, `${row.id} context in GET /api/models`).toBe(row.context);
  expect(entry.tool_call, `${row.id} tool_call in GET /api/models`).toBe(row.tool_call);
  if (row.reasoning !== undefined) {
    expect(entry.reasoning, `${row.id} reasoning in GET /api/models`).toBe(row.reasoning);
  }
  expect(
    entry.modalities?.input?.includes("image") ?? false,
    `${row.id} vision (image input) in GET /api/models`,
  ).toBe(row.vision);
  if (row.cost_input !== undefined && row.cost_output !== undefined) {
    expect(entry.cost?.input ?? 0, `${row.id} input price in GET /api/models`).toBeCloseTo(
      row.cost_input,
      9,
    );
    expect(entry.cost?.output ?? 0, `${row.id} output price in GET /api/models`).toBeCloseTo(
      row.cost_output,
      9,
    );
  }
}

test("SCN-IDN-001 Catalog model identity survives shared bare model ids", async ({
  page,
  request,
}, testInfo) => {
  test.setTimeout(300_000);
  const oracle = loadIdentityOracle();
  const catalog = loadCatalogSnapshot();
  const custom = oracle.customProviderCollisionConfig;
  const expectedTotal = oracle.expectedFromCatalog.providerModelPairs + custom.extraPairs;
  const frozenRows: FrozenModelRow[] = [...oracle.sampleRows, ...custom.customProviderModels];

  await test.step("the frozen oracle agrees with the committed catalog snapshot", async () => {
    const pairs = catalog.flatMap((provider) =>
      (provider.models ?? []).map((model) => ({ providerId: provider.id, modelId: model.id })),
    );
    const providersPerBareId = new Map<string, Set<string>>();
    for (const pair of pairs) {
      const providers = providersPerBareId.get(pair.modelId) ?? new Set<string>();
      providers.add(pair.providerId);
      providersPerBareId.set(pair.modelId, providers);
    }
    expect(pairs.length, "provider/model pairs in the snapshot").toBe(
      oracle.expectedFromCatalog.providerModelPairs,
    );
    expect(providersPerBareId.size, "distinct bare model ids").toBe(
      oracle.expectedFromCatalog.distinctBareModelIds,
    );
    expect(
      [...providersPerBareId.values()].filter((providers) => providers.size >= 2).length,
      "bare ids listed by two or more providers",
    ).toBe(oracle.expectedFromCatalog.bareIdsListedByTwoOrMoreProviders);
    expect(
      pairs.filter((pair) => pair.modelId.includes("/")).length,
      "model ids containing a slash",
    ).toBe(oracle.expectedFromCatalog.modelIdsContainingSlash);
    for (const [bareId, providers] of Object.entries(oracle.sharedBareIdSamples)) {
      expect([...(providersPerBareId.get(bareId) ?? [])].sort(), `providers of ${bareId}`).toEqual(
        [...providers].sort(),
      );
    }
  });

  await test.step("the runtime's GET /api/models agrees with the frozen rows", async () => {
    const response = await request.get("/api/models");
    expect(response.ok(), `GET /api/models returned ${response.status()}`).toBeTruthy();
    const body = await response.text();
    const api = JSON.parse(body) as ApiModels;
    expect(countApiPairs(api), "provider/model pairs in GET /api/models").toBe(expectedTotal);
    for (const row of frozenRows) expectApiEntryMatches(api, row);
    await testInfo.attach("api-models-digest.json", {
      contentType: "application/json",
      body: JSON.stringify({
        sha256: crypto.createHash("sha256").update(body).digest("hex"),
        pairs: countApiPairs(api),
        providers: Object.keys(api).length,
      }),
    });
  });

  await page.goto("/admin/models");

  await test.step("the page reports every provider's model, not one per bare id", async () => {
    const counts = await waitForCatalogTotal(page, expectedTotal);
    expect(counts.shown, "rows shown before any filter").toBe(expectedTotal);
    // The page counts providers from the rows it holds (providers with no models are
    // absent) or from the provider registry (345 = 343 catalog providers + 2 custom).
    const providersWithModels =
      catalog.filter((provider) => (provider.models ?? []).length > 0).length + 2;
    expect([providersWithModels, catalog.length + 2]).toContain(counts.providers);
    // A bare-id key would collapse the catalog to its distinct bare ids plus the
    // custom ids that are not already bare ids.
    expect(counts.total).toBeGreaterThan(oracle.expectedFromCatalog.distinctBareModelIds + 100);
  });

  await test.step("each frozen row shows its own provider, context, capabilities and price", async () => {
    for (const row of frozenRows) await expectCatalogRow(page, row);
  });

  await test.step("the two idn-shared-model rows are both listed, with their own values", async () => {
    await filterCatalog(page, "idn-shared-model");
    await expect
      .poll(async () => (await readCatalogCounts(page))?.shown, {
        message: "searching the shared bare id lists one row per custom provider",
      })
      .toBe(2);
    await expect(page.getByTestId("compare-checkbox-idn-gw-a/idn-shared-model")).toHaveCount(1);
    await expect(page.getByTestId("compare-checkbox-idn-gw-b/idn-shared-model")).toHaveCount(1);
  });

  await test.step("clearing the filter restores the whole catalog", async () => {
    await filterCatalog(page, "");
    await expect
      .poll(async () => (await readCatalogCounts(page))?.shown, {
        message: "the unfiltered list shows every model again",
      })
      .toBe(expectedTotal);
  });
});

test("SCN-IDN-001 negative control: a snapshot record missing model_id is rejected visibly", async ({
  page,
}) => {
  test.setTimeout(300_000);
  const oracle = loadIdentityOracle();
  const expectedTotal =
    oracle.expectedFromCatalog.providerModelPairs +
    oracle.customProviderCollisionConfig.extraPairs;
  const witnessRows: FrozenModelRow[] = [
    ...oracle.sampleRows.slice(0, 3),
    ...oracle.customProviderCollisionConfig.customProviderModels,
  ];

  // Declared substituted boundaries (the only ones in this file). The page picks its
  // realtime adapter from GET /api/config/persistence: an embedded store uses
  // /api/uar/sync/stream, a remote store uses /api/live and the per-topic snapshot
  // route (frontend/src/entities/sync.ts). The harness reports mode "remote" so the page
  // takes the /api/live path against the real runtime, then fulfils the one response
  // GET /api/live/models/snapshot with the real snapshot minus the model_id of one record.
  // Every other request reaches the real server.
  let persistenceRewritten = false;
  let strippedRecordId: string | null = null;
  let strippedProviderId: string | null = null;
  let snapshotsDelivered = 0;

  await page.route("**/api/config/persistence", async (route) => {
    const response = await route.fetch();
    const info = (await response.json()) as Record<string, unknown>;
    persistenceRewritten = true;
    await route.fulfill({
      status: response.status(),
      contentType: "application/json",
      body: JSON.stringify({ ...info, mode: "remote" }),
    });
  });

  await page.route("**/api/live/models/snapshot", async (route) => {
    const response = await route.fetch();
    const snapshot = (await response.json()) as {
      records?: Array<Record<string, unknown>>;
    };
    const records = snapshot.records ?? [];
    // Strip a record whose provider differs from the first record's, so a build that rejects
    // the snapshot on record 0 for any reason names the wrong provider and fails the error check.
    const firstProvider = records[0]?.provider_id;
    let index = -1;
    for (let position = records.length - 1; position > 0; position -= 1) {
      const candidate = records[position];
      if (typeof candidate.model_id === "string" && candidate.provider_id !== firstProvider) {
        index = position;
        break;
      }
    }
    if (index < 1) {
      throw new Error(
        "The real models snapshot has no record with a model_id from a second provider to strip.",
      );
    }
    const broken: Record<string, unknown> = { ...records[index] };
    delete broken.model_id;
    strippedRecordId = typeof broken.id === "string" ? broken.id : null;
    strippedProviderId = typeof broken.provider_id === "string" ? broken.provider_id : null;
    snapshotsDelivered += 1;
    await route.fulfill({
      status: response.status(),
      contentType: "application/json",
      body: JSON.stringify({
        ...snapshot,
        records: records.map((record, position) => (position === index ? broken : record)),
      }),
    });
  });

  await page.goto("/admin/models");
  await waitForCatalogTotal(page, expectedTotal);

  await test.step("the harness's snapshot response was actually delivered to the page", async () => {
    await expect
      .poll(() => snapshotsDelivered, {
        message:
          "the page fetched GET /api/live/models/snapshot; without that the control proves nothing",
        timeout: 60_000,
      })
      .toBeGreaterThan(0);
    expect(persistenceRewritten, "the persistence-mode response was rewritten").toBe(true);
    expect(strippedRecordId, "a snapshot record lost its model_id").not.toBeNull();
    expect(strippedProviderId, "the stripped record names its provider").not.toBeNull();
  });

  await test.step("an ingestion error is visible", async () => {
    const statusBar = page
      .locator("div")
      .filter({ hasText: /^ERR-MODELS/ })
      .filter({ hasText: /model_id|ingest/i });
    const ingestionError = page
      .locator(
        '[role="alert"], [data-testid="models-ingestion-error"], [data-testid="model-catalog-error"]',
      )
      .filter({ hasText: /model_id|ingest/i })
      .or(statusBar)
      .first();
    await expect(
      ingestionError,
      "a visible error names the rejected snapshot record (missing model_id)",
    ).toBeVisible({ timeout: 30_000 });
    await expect(
      ingestionError,
      "the error names the provider of the stripped record, not the first record's",
    ).toContainText(`'${strippedProviderId ?? ""}'`);
  });

  await test.step("the rows are unchanged", async () => {
    const counts = await readCatalogCounts(page);
    expect(counts?.total, "models in the catalog after the rejected snapshot").toBe(expectedTotal);
    expect(counts?.shown, "models shown after the rejected snapshot").toBe(expectedTotal);
    for (const row of witnessRows) await expectCatalogRow(page, row);
    if (strippedRecordId) {
      await filterCatalog(page, strippedRecordId);
      await expect(
        page.getByTestId(`compare-checkbox-${strippedRecordId}`),
        `the row for the rejected record ${strippedRecordId} is still listed once`,
      ).toHaveCount(1);
    }
  });
});
