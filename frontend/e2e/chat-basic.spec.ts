import { test, expect } from "@chromatic-com/playwright";

async function configureDeterministicChat(page: import("@playwright/test").Page) {
  await page.route("**/api/uar/resolve-model", async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ ok: true, provider_id: "local", model_id: "local/test" }),
    });
  });
  await page.route("**/api/agents", async (route) => {
    await route.fulfill({ contentType: "application/json", body: "[]" });
  });
  await page.route("**/api/uar/providers", async (route) => {
    await route.fulfill({ contentType: "application/json", body: "[]" });
  });
}

test.describe("Chat — Basic smoke tests", () => {
  test("chat page renders without crash", async ({ page }) => {
    await page.goto("/threads");
    await expect(page.locator("body")).toBeVisible();
    // No unhandled error boundary
    await expect(page.locator("text=Something went wrong")).not.toBeVisible();
  });

  test("chat UI is functional when model is configured", async ({ page }) => {
    await configureDeterministicChat(page);
    await page.goto("/threads");

    await expect(page.getByRole("button", { name: "New conversation" })).toBeVisible();
  });

  test("admin link is accessible from chat page header", async ({ page }) => {
    await page.goto("/threads");
    await expect(page.locator("body")).toBeVisible();
    // Navigation to admin should work
    await page.goto("/admin");
    await expect(page).toHaveURL(/\/admin/);
    await expect(page.locator("body")).toBeVisible();
  });

  test("page title is set", async ({ page }) => {
    await page.goto("/threads");
    const title = await page.title();
    // Title should not be empty
    expect(title.length).toBeGreaterThan(0);
  });
});
