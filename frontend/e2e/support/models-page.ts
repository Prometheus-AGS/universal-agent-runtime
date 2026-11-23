/**
 * Reads the Models page the way an operator sees it: the count line, the search box
 * and one catalog row at a time. Every locator here exists on the pre-change page
 * (`models-count`, `compare-checkbox-<key>`, the search placeholder) and is kept by
 * the change (task 5.6), so the identity scenario can also run against the baseline.
 */
import { expect, type Locator, type Page } from "@playwright/test";
import type { FrozenModelRow } from "./frozen-fixtures";

export interface CatalogCounts {
  /** Rows matching the current filter. */
  shown: number;
  /** Rows in the catalog. */
  total: number;
  /** Providers the page reports. */
  providers: number;
}

const COUNT_LINE = /([\d,]+)\s+of\s+([\d,]+)\s+in catalog\s*·\s*([\d,]+)\s+providers/;

function toInt(text: string): number {
  return Number.parseInt(text.replace(/,/g, ""), 10);
}

/** Parses the header count line; null while the page has not rendered it yet. */
export async function readCatalogCounts(page: Page): Promise<CatalogCounts | null> {
  const line = page.getByTestId("models-count").locator("..");
  if ((await line.count()) === 0) return null;
  const match = COUNT_LINE.exec((await line.innerText()).replace(/\s+/g, " "));
  if (!match) return null;
  return { shown: toInt(match[1]), total: toInt(match[2]), providers: toInt(match[3]) };
}

/** Waits for the catalog to finish loading and reports exactly `total` models. */
export async function waitForCatalogTotal(page: Page, total: number): Promise<CatalogCounts> {
  await expect
    .poll(async () => (await readCatalogCounts(page))?.total, {
      message: `the Models page reports ${total} models in the catalog`,
      timeout: 90_000,
      intervals: [250, 500, 1000],
    })
    .toBe(total);
  const counts = await readCatalogCounts(page);
  if (!counts) throw new Error("The Models count line disappeared after loading.");
  return counts;
}

export async function filterCatalog(page: Page, query: string): Promise<void> {
  await page.getByPlaceholder(/search models/i).fill(query);
}

/** The row container that holds the compare checkbox for `key`. */
export function catalogRow(page: Page, key: string): Locator {
  const checkbox = page.getByTestId(`compare-checkbox-${key}`);
  return page.locator("div").filter({ has: checkbox }).last();
}

/**
 * Filters the list to one key and compares what the row shows with the frozen row:
 * exactly one row exists for the key, it names the key whole (a slash in the model
 * id stays intact), and its context, capability badges and price match.
 */
export async function expectCatalogRow(page: Page, row: FrozenModelRow): Promise<void> {
  await filterCatalog(page, row.id);
  const checkbox = page.getByTestId(`compare-checkbox-${row.id}`);
  await expect(checkbox, `exactly one catalog row is keyed ${row.id}`).toHaveCount(1);
  const element = catalogRow(page, row.id);
  await expect(element, `row ${row.id} names its own provider-qualified key`).toContainText(row.id);

  const contextLabel = `${Math.round(row.context / 1000)}k ctx`;
  await expect(
    element.getByText(contextLabel, { exact: true }),
    `row ${row.id} shows context ${row.context}`,
  ).toHaveCount(1);

  const badges: Array<[string, boolean | undefined]> = [
    ["tools", row.tool_call],
    ["reasoning", row.reasoning],
    ["vision", row.vision],
  ];
  for (const [badge, expected] of badges) {
    if (expected === undefined) continue;
    await expect(
      element.getByText(badge, { exact: true }),
      `row ${row.id} ${expected ? "shows" : "does not show"} the ${badge} badge`,
    ).toHaveCount(expected ? 1 : 0);
  }

  if (row.cost_input !== undefined && row.cost_output !== undefined) {
    if (row.cost_input > 0) {
      const price = `$${row.cost_input.toFixed(2)}/$${row.cost_output.toFixed(2)} per 1M`;
      await expect(
        element.getByText(price, { exact: true }),
        `row ${row.id} shows the price ${price}`,
      ).toHaveCount(1);
    } else {
      await expect(
        element.getByText(/per 1M/),
        `row ${row.id} is free and shows no price`,
      ).toHaveCount(0);
    }
  }
}
