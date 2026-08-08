import { test, expect } from "@chromatic-com/playwright";

async function configureNoModel(page: import("@playwright/test").Page) {
  await page.route("**/api/uar/resolve-model", async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({ ok: false, error: "No model configured" }),
    });
  });
}

test.describe("Chat — No-model guard", () => {
  test("chat page loads without crashing", async ({ page }) => {
    await page.goto("/threads");
    await expect(page.locator("body")).toBeVisible();
    // URL should be /threads (not a 404 or error)
    await expect(page).toHaveURL(/\/threads/);
  });

  test("chat page shows the no-model guard when resolution fails", async ({ page }) => {
    await configureNoModel(page);
    await page.goto("/threads");
    await expect(page.getByRole("heading", { name: "No Model Configured" })).toBeVisible();
  });

  test("no-model guard CTA navigates to admin", async ({ page }) => {
    await configureNoModel(page);
    await page.goto("/threads");
    await page.getByRole("button", { name: "Configure Provider" }).click();
    await expect(page).toHaveURL(/\/admin/);
  });
});
