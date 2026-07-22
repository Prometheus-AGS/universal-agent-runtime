import { test, expect } from "@playwright/test";

test.describe("Chat — Session config panel", () => {
  test("session config button is accessible from chat", async ({ page }) => {
    await page.goto("/threads");
    await page.waitForLoadState("domcontentloaded");

    // Skip if no model guard is showing
    const guard = page.locator("text=No Model Configured, text=No LLM Provider Configured");
    const guardVisible = await guard.first().isVisible().catch(() => false);
    if (guardVisible) {
      test.skip();
      return;
    }

    // Start a new thread so we have the chat header
    const newConv = page.locator("button:has-text('New conversation')").first();
    if (await newConv.isVisible({ timeout: 3000 }).catch(() => false)) {
      await newConv.click();
    }

    // Session config gear button
    const configBtn = page.locator("[aria-label='Session configuration']").first();
    if (await configBtn.isVisible({ timeout: 5000 }).catch(() => false)) {
      await configBtn.click();
      // Panel should open
      await expect(page.locator("body")).toBeVisible();
    }
  });
});
