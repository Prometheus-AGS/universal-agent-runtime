import { expect, test } from "@playwright/test";

interface ProviderSetting {
  id: string;
  key: string;
  name: string;
}

interface SettingsRequest {
  method: string;
  path: string;
  status?: number;
}

test("installed settings panels use canonical namespace routes without settings 404s", async ({
  page,
  request,
}) => {
  const settingsRequests: SettingsRequest[] = [];
  const consoleErrors: string[] = [];

  page.on("request", (entry) => {
    const url = new URL(entry.url());
    if (url.origin === "http://127.0.0.1:1906" && url.pathname.startsWith("/api/uar/settings/")) {
      settingsRequests.push({ method: entry.method(), path: url.pathname });
    }
  });
  page.on("response", (entry) => {
    const url = new URL(entry.url());
    const record = [...settingsRequests].reverse().find((candidate) => (
      candidate.method === entry.request().method() && candidate.path === url.pathname
    ));
    if (record) record.status = entry.status();
  });
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });

  const configuredResponse = await request.get("/api/uar/settings/providers");
  expect(configuredResponse.ok(), await configuredResponse.text()).toBeTruthy();
  const configuredProviders = await configuredResponse.json() as ProviderSetting[];
  expect(configuredProviders.length).toBeGreaterThan(0);

  const providerResponsePromise = page.waitForResponse((response) => (
    response.request().method() === "GET"
    && new URL(response.url()).pathname === "/api/uar/settings/providers"
  ));
  await page.goto("/admin/settings", { waitUntil: "domcontentloaded" });
  const providerResponse = await providerResponsePromise;
  expect(providerResponse.ok()).toBeTruthy();

  await expect(page.getByRole("heading", { name: "LLM Providers" })).toBeVisible();
  await expect(page.getByText(`${configuredProviders.length} provider(s) configured`)).toBeVisible();
  await expect(page.getByText("This item wasn’t found. It may have been deleted or moved.")).toHaveCount(0);
  await expect(page.getByText("No providers configured.", { exact: false })).toHaveCount(0);
  for (const provider of configuredProviders) {
    await expect(page.getByText(provider.name, { exact: true })).toBeVisible();
    await expect(page.getByText(provider.key, { exact: true })).toBeVisible();
  }

  const contextResponsePromise = page.waitForResponse((response) => (
    response.request().method() === "GET"
    && new URL(response.url()).pathname === "/api/uar/settings/context-management"
  ));
  await page.getByRole("button", { name: "Context Management" }).click();
  const contextResponse = await contextResponsePromise;
  expect(contextResponse.ok()).toBeTruthy();
  await expect(page.getByRole("heading", { name: "Context Management" })).toBeVisible();
  await expect(page.getByText("This item wasn’t found. It may have been deleted or moved.")).toHaveCount(0);

  expect(settingsRequests.some((entry) => entry.path === "/api/uar/settings/providers")).toBeTruthy();
  expect(settingsRequests.some((entry) => entry.path === "/api/uar/settings/context-management")).toBeTruthy();
  expect(settingsRequests.some((entry) => entry.path === "/api/uar/settings/provider")).toBeFalsy();
  expect(settingsRequests.some((entry) => entry.path.includes("_"))).toBeFalsy();
  expect(settingsRequests.filter((entry) => entry.status === 404)).toEqual([]);
  expect(consoleErrors.filter((entry) => (
    entry.includes("/api/uar/settings/") && entry.includes("404")
  ))).toEqual([]);
});
