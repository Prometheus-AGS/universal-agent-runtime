import { expect, test } from "@playwright/test";

test("configured provider and default model drive routing and its UI decision", async ({
  page,
  request,
}) => {
  await page.goto("/admin/providers");

  const anthropic = page.getByTestId("provider-row-anthropic");
  await expect(anthropic).toBeVisible();
  await anthropic.click();
  await page.getByRole("button", { name: "Configure", exact: true }).click();
  await page.getByLabel("API Key").fill("playwright-secret");
  await page.getByLabel("Base URL Override").fill("http://127.0.0.1:4601/v1");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect(page.getByText("Configured", { exact: true }).first()).toBeVisible();

  const providersResponse = await request.get("/api/uar/providers");
  expect(providersResponse.ok()).toBeTruthy();
  const providers = (await providersResponse.json()) as {
    providers: Array<Record<string, unknown>>;
  };
  const configured = providers.providers.find((provider) => provider.id === "anthropic");
  expect(configured).toBeDefined();

  const updateResponse = await request.put("/api/uar/providers/anthropic", {
    data: {
      ...configured,
      id: "anthropic",
      display_name: "Anthropic",
      base_url: "http://127.0.0.1:4601/v1",
      default_model: "claude-5-haiku",
      models: [{ id: "claude-5-haiku", enabled: true }],
      enabled: true,
    },
  });
  expect(updateResponse.ok(), await updateResponse.text()).toBeTruthy();

  await page.getByRole("button", { name: "Set as default", exact: true }).click();
  await expect(page.getByText("Default", { exact: true }).first()).toBeVisible();

  const resolvedResponse = await request.get("/api/uar/resolve-model");
  expect(resolvedResponse.ok()).toBeTruthy();
  await expect(resolvedResponse.json()).resolves.toMatchObject({
    ok: true,
    provider_id: "anthropic",
    model_id: "claude-5-haiku",
  });

  const routeResponse = await request.post("/api/uar/route", {
    data: { preferred_provider: "anthropic" },
  });
  expect(routeResponse.ok(), await routeResponse.text()).toBeTruthy();
  const route = (await routeResponse.json()) as { model?: string };
  expect(route.model).toMatch(/^anthropic\//);

  await page.goto("/admin/protocols");
  await expect(page.getByText("claude-5-haiku", { exact: true })).toBeVisible({
    timeout: 15_000,
  });
});
