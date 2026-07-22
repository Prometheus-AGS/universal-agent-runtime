import { test, expect } from "@playwright/test";

test.describe("Admin — Providers page", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/admin/providers");
  });

  test("providers page loads and shows header", async ({ page }) => {
    // Wait for page to settle
    await expect(page.locator("body")).toBeVisible();
    // Should contain the Providers heading or nav item
    const heading = page.locator("h2, [data-testid='providers-heading']").filter({ hasText: /providers/i });
    await expect(heading.first()).toBeVisible({ timeout: 15000 });
  });

  test("navigating to providers does not crash the app", async ({ page }) => {
    await expect(page).toHaveURL(/\/admin/);
    // No error boundaries should be visible
    const errorMsg = page.locator("text=Something went wrong");
    await expect(errorMsg).not.toBeVisible();
  });

  test("admin sidebar contains Providers link", async ({ page }) => {
    await page.goto("/admin");
    await expect(page.locator("text=Providers").first()).toBeVisible();
  });
});
