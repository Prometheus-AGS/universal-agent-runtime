import { test, expect } from "@chromatic-com/playwright";

test.describe("Admin — Knowledge page", () => {
  test("knowledge page loads", async ({ page }) => {
    await page.goto("/admin/knowledge");
    await expect(page.locator("body")).toBeVisible();
    await expect(page).toHaveURL(/\/admin/);
  });

  test("admin sidebar contains Knowledge link", async ({ page }) => {
    await page.goto("/admin");
    await expect(
      page.locator("text=Knowledge").first(),
    ).toBeVisible();
  });

  test("knowledge page shows content without crashing", async ({ page }) => {
    await page.goto("/admin/knowledge");
    const errorBoundary = page.locator("text=Something went wrong");
    await expect(errorBoundary).not.toBeVisible();
  });
});
