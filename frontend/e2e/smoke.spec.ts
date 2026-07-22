import { test, expect } from "@playwright/test";

test("admin page loads with Providers nav item", async ({ page }) => {
  await page.goto("/admin");
  await expect(page.locator("text=Providers").first()).toBeVisible();
});

test("chat page loads", async ({ page }) => {
  await page.goto("/threads");
  // The chat page either shows the thread view or no-provider guard
  await expect(page).toHaveURL(/\/threads/);
  // Page should load without crashing
  await expect(page.locator("body")).toBeVisible();
});
