/**
 * SCN-PICK-002: the model picker is searchable and keyboard operable.
 *
 * Actor: an operator choosing a model. Outcome: a fast, searchable choice from the
 * 597 catalog models of nano-gpt. Production entry: the Add Model dialog on
 * /admin/models, with the runtime's real catalog and the browser.
 *
 * Oracle (frozen, never edited): picker-oracle.json through ./support/frozen-fixtures.ts
 * (597 options, the first and last ten ids in raw-id order, search counts for
 * claude/gpt/llama/qwen/zzzz-no-match, and the 200 ms long-task threshold).
 * Comparator: equality, and below the threshold.
 *
 * The pre-change dialog's model control is a plain select with no search box, so the
 * first thing the main test needs after opening it (a visible search field) is exactly
 * what the baseline lacks; the test fails there, not in setup.
 *
 * Setup fidelity, stated plainly: picker-config.yaml lists nano-gpt with `models: []`,
 * but the runtime fills an empty model list of a catalog provider with every catalog
 * model, enabled (src/llm/registry.rs:754-755 via enrich_provider_config, called at
 * seeding in seed_from_configs, :397, and again on PUT in src/uar/api/providers.rs:278).
 * The dialog offers catalog models that are not already enabled, so with that default it
 * would offer none. The test therefore puts the provider into the state the oracle
 * describes, "no enabled catalog model", through the runtime's own provider API: one
 * disabled placeholder model that is not in the catalog. See prepareNanoGpt below.
 *
 * No credentials, tokens or Authorization headers are used anywhere in this file.
 */
import { expect, test, type APIRequestContext, type Page } from "@playwright/test";
import {
  loadCatalogSnapshot,
  loadPickerOracle,
  type PickerOracle,
} from "./support/frozen-fixtures";
import {
  installLongTaskObserver,
  readLongTasks,
  resetLongTasks,
  type LongTaskReading,
} from "./support/long-tasks";

const PLACEHOLDER_MODEL_ID = "uar-picker-oracle-placeholder";
const SETTLE = { timeout: 30_000 } as const;

interface NanoGptCatalogModel {
  id: string;
  label: string;
}

/** nano-gpt's catalog models in raw-id (code unit) order, from the sha-pinned snapshot. */
function nanoGptCatalogModels(oracle: PickerOracle): NanoGptCatalogModel[] {
  const provider = loadCatalogSnapshot().find((entry) => entry.id === oracle.provider.id);
  expect(provider, `${oracle.provider.id} is in the committed catalog snapshot`).toBeDefined();
  return (provider?.models ?? [])
    .map((model) => ({ id: model.id, label: model.name || model.id }))
    .sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
}

/** Models whose display name or raw id contains the trimmed, lower-cased query. */
function matching(models: NanoGptCatalogModel[], query: string): NanoGptCatalogModel[] {
  const needle = query.trim().toLowerCase();
  return models.filter(
    (model) =>
      model.label.toLowerCase().includes(needle) || model.id.toLowerCase().includes(needle),
  );
}

/**
 * Leaves nano-gpt configured with no enabled catalog model (see the header). Idempotent:
 * it replaces the model list on every call, so repeated samples start from the same state.
 */
async function prepareNanoGpt(request: APIRequestContext, oracle: PickerOracle): Promise<void> {
  const providerId = oracle.provider.id;
  const listed = await request.get("/api/uar/providers");
  expect(listed.ok(), `GET /api/uar/providers returned ${listed.status()}`).toBeTruthy();
  const { providers } = (await listed.json()) as { providers: Array<Record<string, unknown>> };
  const provider = providers.find((entry) => entry.id === providerId);
  expect(
    provider,
    `${providerId} is configured: the runtime loaded the frozen picker config through CONFIG_FILE`,
  ).toBeDefined();

  const updated = await request.put(`/api/uar/providers/${providerId}`, {
    data: {
      ...provider,
      id: providerId,
      default_model: null,
      models: [
        {
          id: PLACEHOLDER_MODEL_ID,
          display_name: "Picker oracle placeholder (disabled, not a catalog model)",
          enabled: false,
        },
      ],
    },
  });
  expect(updated.ok(), `PUT /api/uar/providers/${providerId}: ${await updated.text()}`).toBeTruthy();

  const catalog = await request.get("/api/models");
  expect(catalog.ok(), `GET /api/models returned ${catalog.status()}`).toBeTruthy();
  const models = ((await catalog.json()) as Record<string, { models?: Record<string, unknown> }>)[
    providerId
  ]?.models;
  const catalogIds = Object.keys(models ?? {}).filter((id) => id !== PLACEHOLDER_MODEL_ID);
  expect(catalogIds.length, `${providerId} catalog models served by GET /api/models`).toBe(
    oracle.provider.catalogModelCount,
  );
}

/** Opens Add Model, chooses the provider, and returns the model control's trigger. */
async function openAddModelDialog(page: Page, oracle: PickerOracle) {
  await page.goto("/admin/models");
  const addButton = page.getByTestId("models-add-button");
  await expect(addButton, "Add model is enabled once providers have loaded").toBeEnabled({
    timeout: 90_000,
  });
  await addButton.click();

  const providerTrigger = page.getByTestId("add-model-provider");
  await expect(providerTrigger).toBeVisible(SETTLE);
  await providerTrigger.click();
  await page
    .getByRole("option", { name: oracle.provider.display_name, exact: true })
    .click();
  await expect(providerTrigger).toContainText(oracle.provider.display_name, SETTLE);

  const modelTrigger = page.getByTestId("add-model-model");
  await expect(modelTrigger, "the model control is enabled for the chosen provider").toBeEnabled(
    SETTLE,
  );
  return modelTrigger;
}

const searchField = (page: Page) => page.getByLabel("Search model", { exact: true });
const options = (page: Page) => page.getByTestId("model-option");
const countStatus = (page: Page) => page.getByTestId("picker-status");

function optionIds(page: Page): Promise<string[]> {
  return options(page).evaluateAll((elements) =>
    elements.map((element) => element.getAttribute("data-model-id") ?? ""),
  );
}

test("SCN-PICK-002 Model picker is searchable and keyboard operable", async ({
  page,
  request,
}, testInfo) => {
  test.setTimeout(240_000);
  const oracle = loadPickerOracle();
  const models = nanoGptCatalogModels(oracle);
  const providerPrefix = `${oracle.provider.id}/`;
  const longTaskReadings: LongTaskReading[] = [];

  await test.step("the frozen oracle agrees with the committed catalog snapshot", async () => {
    expect(models.length, "nano-gpt models in the snapshot").toBe(oracle.provider.catalogModelCount);
    expect(models.slice(0, 10).map((model) => model.id), "first ten ids in raw order").toEqual(
      oracle.firstTenSortedIds,
    );
    expect(models.slice(-10).map((model) => model.id), "last ten ids in raw order").toEqual(
      oracle.lastTenSortedIds,
    );
    for (const [query, expected] of Object.entries(oracle.searchCases)) {
      expect(matching(models, query).length, `snapshot matches for "${query}"`).toBe(expected);
    }
  });

  await prepareNanoGpt(request, oracle);
  await installLongTaskObserver(page);

  await test.step("the model control opens a searchable list of every catalog model, in order", async () => {
    const modelTrigger = await openAddModelDialog(page, oracle);
    await resetLongTasks(page);

    await modelTrigger.click();
    await expect(
      searchField(page),
      "the model picker offers a search field (the pre-change control is a plain select)",
    ).toBeVisible(SETTLE);

    await expect(countStatus(page)).toHaveText(`${oracle.provider.catalogModelCount} models`, SETTLE);
    await expect(options(page), "one option per catalog model").toHaveCount(
      oracle.provider.catalogModelCount,
      SETTLE,
    );
    const ids = await optionIds(page);
    expect(ids, "option order is raw model-id order").toEqual(
      models.map((model) => `${providerPrefix}${model.id}`),
    );
    expect(ids.slice(0, 10), "first ten options").toEqual(
      oracle.firstTenSortedIds.map((id) => `${providerPrefix}${id}`),
    );
    expect(ids.slice(-10), "last ten options").toEqual(
      oracle.lastTenSortedIds.map((id) => `${providerPrefix}${id}`),
    );
  });

  await test.step("typing narrows the list to the frozen counts", async () => {
    const search = searchField(page);
    for (const [query, expected] of Object.entries(oracle.searchCases)) {
      await search.fill(query);
      await expect(countStatus(page), `status for "${query}"`).toHaveText(
        `${expected} ${expected === 1 ? "model" : "models"}`,
        SETTLE,
      );
      await expect(options(page), `options for "${query}"`).toHaveCount(expected, SETTLE);
      expect(await optionIds(page), `options for "${query}" are the matching catalog models`).toEqual(
        matching(models, query).map((model) => `${providerPrefix}${model.id}`),
      );
      if (expected === 0) {
        await expect(page.getByText("No matching models.")).toBeVisible(SETTLE);
      }
    }
    await search.fill("");
    await expect(countStatus(page)).toHaveText(`${oracle.provider.catalogModelCount} models`, SETTLE);
    longTaskReadings.push(await readLongTasks(page));
  });

  await test.step("a fresh dialog is operable from the keyboard alone", async () => {
    const modelTrigger = await openAddModelDialog(page, oracle);
    await resetLongTasks(page);

    await modelTrigger.focus();
    await page.keyboard.press("Enter");
    await expect(searchField(page), "Enter on the focused control opens the picker").toBeVisible(
      SETTLE,
    );
    await expect(searchField(page), "the search field takes focus on open").toBeFocused();

    await page.keyboard.type("claude");
    const claude = matching(models, "claude");
    expect(claude.length, "frozen count for claude").toBe(oracle.searchCases.claude);
    await expect(countStatus(page)).toHaveText(`${oracle.searchCases.claude} models`, SETTLE);
    await expect(options(page)).toHaveCount(oracle.searchCases.claude, SETTLE);

    const highlighted = page.locator('[data-testid="model-option"][data-highlighted]');
    await page.keyboard.press("ArrowDown");
    await expect(highlighted, "one option is highlighted after ArrowDown").toHaveCount(1, SETTLE);
    const chosenId = await highlighted.getAttribute("data-model-id");
    const claudeIds = claude.map((model) => `${providerPrefix}${model.id}`);
    expect(
      claudeIds.slice(0, 2),
      "the highlight sits on the first or second result for the query",
    ).toContain(chosenId);
    const chosen = claude.find((model) => `${providerPrefix}${model.id}` === chosenId);
    expect(chosen, `highlighted option ${chosenId} is a claude match`).toBeDefined();

    await page.keyboard.press("Enter");
    await expect(searchField(page), "Enter selects the option and closes the list").toBeHidden(
      SETTLE,
    );
    await expect(modelTrigger, "the control shows the keyboard-chosen model").toContainText(
      chosen?.label ?? "",
      SETTLE,
    );
    await expect(
      page.getByTestId("add-model-confirm"),
      "Add model is enabled once a model is chosen",
    ).toBeEnabled(SETTLE);
    longTaskReadings.push(await readLongTasks(page));
  });

  await test.step(`no long task reaches ${oracle.thresholds.maxLongTaskMs} ms while using the picker`, async () => {
    const tasks = longTaskReadings.flatMap((reading) => reading.tasks);
    await testInfo.attach("long-tasks.json", {
      contentType: "application/json",
      body: JSON.stringify({ thresholdMs: oracle.thresholds.maxLongTaskMs, readings: longTaskReadings }),
    });
    expect(
      longTaskReadings.every((reading) => reading.supported),
      "the browser reports long tasks, so an empty list means none occurred",
    ).toBe(true);
    const longest = tasks.reduce((max, task) => Math.max(max, task.duration), 0);
    expect(longest, `longest long task in ms (${tasks.length} recorded)`).toBeLessThan(
      oracle.thresholds.maxLongTaskMs,
    );
  });
});

test("SCN-PICK-002 negative control: a query that matches nothing shows the empty state", async ({
  page,
  request,
}) => {
  test.setTimeout(240_000);
  const oracle = loadPickerOracle();
  const noMatch = "zzzz-no-match";
  expect(oracle.searchCases[noMatch], "frozen count for the no-match query").toBe(0);

  await prepareNanoGpt(request, oracle);
  const modelTrigger = await openAddModelDialog(page, oracle);
  await modelTrigger.click();
  await expect(searchField(page)).toBeVisible(SETTLE);

  // A real query first, so the empty state below is a change and not the initial view.
  await searchField(page).fill("claude");
  await expect(countStatus(page)).toHaveText(`${oracle.searchCases.claude} models`, SETTLE);

  await searchField(page).fill(noMatch);
  await expect(countStatus(page), "no model matches the query").toHaveText("0 models", SETTLE);
  await expect(options(page)).toHaveCount(0, SETTLE);
  await expect(page.getByText("No matching models."), "the empty state is shown").toBeVisible(
    SETTLE,
  );
  await expect(
    page.getByTestId("add-model-confirm"),
    "nothing is chosen, so Add model stays disabled",
  ).toBeDisabled();
});
