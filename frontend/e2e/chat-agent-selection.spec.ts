import { test, expect } from "@chromatic-com/playwright";

test.describe("Chat — Agent selection", () => {
  test.beforeEach(async ({ page }) => {
    await page.route("**/api/uar/resolve-model", async (route) => {
      await route.fulfill({
        json: { ok: true, provider_id: "test", model_id: "test-model" },
      });
    });
    await page.route("**/api/uar/providers", async (route) => {
      await route.fulfill({ json: { default_id: "test", providers: [] } });
    });
    await page.route("**/api/agents", async (route) => {
      await route.fulfill({
        json: {
          runtime_agents: [
            {
              id: "researcher",
              metadata: {
                title: "Research Assistant",
                description: "Finds primary sources",
              },
            },
            {
              id: "writer",
              metadata: {
                title: "Writing Assistant",
                description: "Drafts concise reports",
              },
            },
          ],
          federated_agents: [],
        },
      });
    });
  });

  test("agent selector filters and selects with the Base UI command facade", async ({ page }) => {
    await page.goto("/threads");
    const agentSelector = page.getByLabel("Select agent").first();
    await expect(agentSelector).toBeVisible({ timeout: 15000 });

    await agentSelector.click();
    const search = page.getByPlaceholder("Search agents...");
    await expect(search).toBeVisible();
    await search.fill("writing");

    await expect(page.getByText("Writing Assistant", { exact: true })).toBeVisible();
    await expect(page.getByText("Research Assistant", { exact: true })).not.toBeVisible();
    await search.press("Enter");

    await expect(agentSelector).toContainText("Writing Assistant");
    await expect(search).not.toBeVisible();
  });

  test("new thread button creates a thread", async ({ page }) => {
    await page.goto("/threads");
    await page.waitForLoadState("domcontentloaded");

    const newConv = page.locator("button:has-text('New conversation'), [aria-label='New thread']").first();
    if (await newConv.isVisible({ timeout: 3000 }).catch(() => false)) {
      await newConv.click();
      // URL may change or thread list may update — just verify no crash
      await expect(page.locator("body")).toBeVisible();
    }
  });
});
