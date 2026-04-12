import { test, expect } from "@playwright/test";

test.describe("Admin — Skills page", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/admin/skills");
  });

  test("skills page loads", async ({ page }) => {
    await expect(page.locator("body")).toBeVisible();
    await expect(page).toHaveURL(/\/admin/);
  });

  test("admin sidebar contains Skills link", async ({ page }) => {
    await page.goto("/admin");
    await expect(page.locator("text=Skills").first()).toBeVisible();
  });

  test("skills list or empty state is visible", async ({ page }) => {
    const content = page.locator(
      "text=No skills configured, [data-testid='skills-list'], button:has-text('New Skill')",
    );
    await expect(content.first()).toBeVisible({ timeout: 8000 });
  });

  test("new skill dialog includes model override selector", async ({ page }) => {
    const newBtn = page.locator("button:has-text('New Skill')").first();
    await expect(newBtn).toBeVisible({ timeout: 8000 });
    await newBtn.click();

    // Dialog should appear
    const dialog = page.locator("[role='dialog']").first();
    await expect(dialog).toBeVisible({ timeout: 5000 });

    // Model override label should be visible
    const modelLabel = dialog.locator("text=Model override");
    await expect(modelLabel).toBeVisible({ timeout: 5000 });
  });

  test("new skill dialog has required title field", async ({ page }) => {
    const newBtn = page.locator("button:has-text('New Skill')").first();
    await newBtn.click();

    const dialog = page.locator("[role='dialog']").first();
    await expect(dialog).toBeVisible({ timeout: 5000 });

    const titleInput = dialog.locator("input#skill-title, input[placeholder='Customer Success Coach']").first();
    await expect(titleInput).toBeVisible({ timeout: 3000 });
  });
});
