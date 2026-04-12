import { test, expect } from "@playwright/test";

test.describe("Admin — Tools page", () => {
  test("tools page loads", async ({ page }) => {
    await page.goto("/admin/tools");
    await expect(page.locator("body")).toBeVisible();
    await expect(page).toHaveURL(/\/admin/);
  });

  test("admin sidebar contains Tools link", async ({ page }) => {
    await page.goto("/admin");
    await expect(page.locator("text=Tools").first()).toBeVisible();
  });

  test("tools page shows content or empty state without crashing", async ({ page }) => {
    await page.goto("/admin/tools");
    // Should not show an unhandled error boundary
    const errorBoundary = page.locator("text=Something went wrong");
    await expect(errorBoundary).not.toBeVisible();
  });
});
