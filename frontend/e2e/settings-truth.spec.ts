import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test, type APIRequestContext, type Page } from "@playwright/test";

/**
 * SCN-SET-003 (openspec change uiux-settings-truthful-status-and-removals, task 5.3).
 *
 * Actor: console operator. Outcome: the system default provider changes through one live
 * control, takes effect at once, and has no second, boot-only writer.
 *
 * Run through `playwright.installed-settings-truth.config.ts`, which boots the real UAR binary
 * with two configured providers (`openai` on one `stub-llm` listener, `anthropic` on another).
 * No test here sends an Authorization or admin-key header; authentication is not a scenario step.
 *
 * Oracle (frozen): `fixtures/removed-settings.json` key `defaultProviderWrite`, inlined below so
 * the spec also runs from a worktree of a commit that predates that file. When the file is
 * present the inlined values are cross-checked against it.
 */

const DEFAULT_PROVIDER_KEY = "llm.default_provider";
const SETTINGS_ROW_URL = `/api/uar/settings/${DEFAULT_PROVIDER_KEY}`;
const PROVIDERS_URL = "/api/uar/providers";
/** The two providers the config boots with (see the config file header). */
const CONFIGURED_PROVIDER_IDS = ["openai", "anthropic"] as const;

const DEFAULT_PROVIDER_WRITE = {
  status: 422,
  body: { error: "unsupported_setting", key: DEFAULT_PROVIDER_KEY },
  messageContains: "POST /api/uar/providers/{id}/default",
} as const;

interface ProviderView {
  id: string;
  enabled: boolean;
}

interface ProvidersResponse {
  providers: ProviderView[];
  default_id: string | null;
}

interface SettingRow {
  key: string;
  data: unknown;
  saved: unknown;
  effective: unknown;
  revision: string;
  application_status: string;
}

async function readProviders(request: APIRequestContext): Promise<ProvidersResponse> {
  const response = await request.get(PROVIDERS_URL);
  expect(response.status(), await response.text()).toBe(200);
  return (await response.json()) as ProvidersResponse;
}

async function readDefaultRow(request: APIRequestContext): Promise<SettingRow> {
  const response = await request.get(SETTINGS_ROW_URL);
  expect(response.status(), await response.text()).toBe(200);
  const row = (await response.json()) as SettingRow;
  expect(row.key).toBe(DEFAULT_PROVIDER_KEY);
  return row;
}

/** Both fixture providers are registered and enabled; returns the current default id. */
async function requireTwoConfiguredProviders(request: APIRequestContext): Promise<string> {
  const providers = await readProviders(request);
  for (const id of CONFIGURED_PROVIDER_IDS) {
    const provider = providers.providers.find((candidate) => candidate.id === id);
    expect(provider, `provider '${id}' is registered`).toBeDefined();
    expect(provider?.enabled, `provider '${id}' is enabled`).toBe(true);
  }
  expect(typeof providers.default_id).toBe("string");
  expect((providers.default_id ?? "").length).toBeGreaterThan(0);
  return providers.default_id as string;
}

function otherProviderId(currentDefault: string): string {
  const other = CONFIGURED_PROVIDER_IDS.find((id) => id !== currentDefault);
  expect(other, `a configured provider other than '${currentDefault}'`).toBeDefined();
  return other as string;
}

/** The refusal of a settings-API write to `llm.default_provider`: exact status and body fields. */
async function expectDefaultProviderWriteRefused(
  request: APIRequestContext,
  value: string,
  expectedRevision: string,
): Promise<void> {
  const response = await request.put(SETTINGS_ROW_URL, {
    data: { value, expected_revision: expectedRevision },
  });
  const text = await response.text();
  expect(response.status(), text).toBe(DEFAULT_PROVIDER_WRITE.status);
  const body = JSON.parse(text) as Record<string, unknown>;
  expect(body.error).toBe(DEFAULT_PROVIDER_WRITE.body.error);
  expect(body.key).toBe(DEFAULT_PROVIDER_WRITE.body.key);
  expect(typeof body.message).toBe("string");
  expect(body.message as string).toContain(DEFAULT_PROVIDER_WRITE.messageContains);
}

async function openProvidersPage(page: Page): Promise<void> {
  await page.goto("/admin/providers");
  await expect(page.getByTestId("providers-heading")).toBeVisible();
}

/** When the frozen fixture is on disk, the inlined oracle must equal it exactly. */
function crossCheckFrozenOracle(): void {
  const fixturePath = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "../../openspec/changes/uiux-settings-truthful-status-and-removals/fixtures/removed-settings.json",
  );
  if (!fs.existsSync(fixturePath)) return;
  const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf8")) as {
    defaultProviderWrite: {
      expectedStatus: number;
      expectedBody: Record<string, string>;
      expectedMessageContains: string;
    };
  };
  expect(fixture.defaultProviderWrite.expectedStatus).toBe(DEFAULT_PROVIDER_WRITE.status);
  expect(fixture.defaultProviderWrite.expectedBody).toEqual(DEFAULT_PROVIDER_WRITE.body);
  expect(fixture.defaultProviderWrite.expectedMessageContains).toBe(
    DEFAULT_PROVIDER_WRITE.messageContains,
  );
}

test("SCN-SET-003 System default has one live control", async ({ page, request }) => {
  crossCheckFrozenOracle();
  const previousDefault = await requireTwoConfiguredProviders(request);
  const target = otherProviderId(previousDefault);

  const authorizationHeaders: Array<string | undefined> = [];
  page.on("request", (entry) => {
    if (new URL(entry.url()).pathname.startsWith("/api/")) {
      authorizationHeaders.push(entry.headers()["authorization"]);
    }
  });

  await openProvidersPage(page);
  const targetRow = page.getByTestId(`provider-row-${target}`);
  const previousRow = page.getByTestId(`provider-row-${previousDefault}`);
  await expect(targetRow).toBeVisible();
  await expect(previousRow).toBeVisible();
  // Before the click the previous default carries the pill and the target does not.
  await expect(previousRow).toContainText("default");
  await expect(targetRow).not.toContainText("default");

  await targetRow.click();
  const setDefault = page.getByRole("button", { name: "Set as default", exact: true });
  await expect(setDefault).toBeVisible();

  const defaultResponse = page.waitForResponse(
    (response) =>
      response.request().method() === "POST" &&
      new URL(response.url()).pathname === `${PROVIDERS_URL}/${target}/default`,
  );
  await setDefault.click();
  expect((await defaultResponse).status()).toBe(200);

  // The page reflects the swap: the target row carries the pill, the previous default lost it,
  // and the detail pane no longer offers the action.
  await expect(targetRow).toContainText("default");
  await expect(previousRow).not.toContainText("default");
  await expect(setDefault).toHaveCount(0);

  // Registry truth: the provider list default is exactly the clicked id, at once.
  const afterSwap = await readProviders(request);
  expect(afterSwap.default_id).toBe(target);

  // The settings API is not a second writer: writing the previous default back is refused,
  // and the registry keeps the clicked default.
  const swapRow = await readDefaultRow(request);
  await expectDefaultProviderWriteRefused(request, previousDefault, swapRow.revision);
  expect((await readProviders(request)).default_id).toBe(target);

  // Row truth: the saved value is the clicked id and the live process reports it in effect.
  const row = await readDefaultRow(request);
  expect(row.data).toBe(target);
  expect(row.saved).toBe(target);
  expect(row.effective).toBe(target);
  expect(row.application_status).toBe("effective");
  expect(row).toEqual(swapRow);

  // No credential travelled with any request the page made.
  expect(authorizationHeaders.length).toBeGreaterThan(0);
  expect(authorizationHeaders.filter((value) => value !== undefined)).toEqual([]);
});

test("SCN-SET-003 negative control", async ({ page, request }) => {
  crossCheckFrozenOracle();
  const currentDefault = await requireTwoConfiguredProviders(request);
  const other = otherProviderId(currentDefault);
  const providersBefore = await readProviders(request);
  const rowBefore = await readDefaultRow(request);

  // A settings PUT of the other provider as llm.default_provider is refused with the frozen
  // 422 body, and neither the registry nor the row moves.
  await expectDefaultProviderWriteRefused(request, other, rowBefore.revision);
  expect(await readProviders(request)).toEqual(providersBefore);
  expect(await readDefaultRow(request)).toEqual(rowBefore);

  // The default route for an unregistered provider id is 404 and changes nothing.
  const unknown = await request.post(`${PROVIDERS_URL}/nosuch/default`);
  const unknownText = await unknown.text();
  expect(unknown.status(), unknownText).toBe(404);
  expect((JSON.parse(unknownText) as { error: string }).error).toContain("nosuch");
  expect(await readProviders(request)).toEqual(providersBefore);
  expect(await readDefaultRow(request)).toEqual(rowBefore);

  // The page agrees: the unchanged default still carries the pill, the other provider does not.
  await openProvidersPage(page);
  await expect(page.getByTestId(`provider-row-${currentDefault}`)).toContainText("default");
  await expect(page.getByTestId(`provider-row-${other}`)).not.toContainText("default");
});
