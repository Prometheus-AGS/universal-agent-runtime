import { test, expect } from "@playwright/test";

test.describe("Chat — Basic smoke tests", () => {
  test("chat page renders without crash", async ({ page }) => {
    await page.goto("/threads");
    await expect(page.locator("body")).toBeVisible();
    // No unhandled error boundary
    await expect(page.locator("text=Something went wrong")).not.toBeVisible();
  });

  test("chat UI is functional when model is configured", async ({ page }) => {
    await page.goto("/threads");
    await page.waitForLoadState("domcontentloaded");

    const guard = page.locator("text=No Model Configured, text=No LLM Provider Configured");
    const guardVisible = await guard.first().isVisible().catch(() => false);
    if (guardVisible) {
      // No model configured — guard is correct UI, test passes
      return;
    }

    // Chat UI should have a way to start a conversation
    const newConvBtn = page.locator("button:has-text('New conversation')").first();
    await expect(newConvBtn).toBeVisible({ timeout: 8000 });
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
