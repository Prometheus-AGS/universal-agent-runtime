import { test, expect } from "@playwright/test";

test.describe("Chat — No-model guard", () => {
  test("chat page loads without crashing", async ({ page }) => {
    await page.goto("/threads");
    await expect(page.locator("body")).toBeVisible();
    // URL should be /threads (not a 404 or error)
    await expect(page).toHaveURL(/\/threads/);
  });

  test("chat page shows either chat UI or no-model guard", async ({ page }) => {
    await page.goto("/threads");

    // Wait for the page to finish loading (model check completes)
    await page.waitForLoadState("domcontentloaded");

    // One of two valid states:
    // 1. No-model guard is shown (no model configured)
    // 2. Normal chat UI is shown (model is configured)
    // Wait for either to appear — the app boots PGlite and the model check
    // asynchronously after domcontentloaded.
    const guardOrChat = page
      .locator("text=No Model Configured, text=No LLM Provider Configured")
      .or(page.locator("button:has-text('New conversation')"))
      .or(page.locator("[aria-label='Send']"))
      .or(page.locator("text=Select a thread"));

    await expect(guardOrChat.first()).toBeVisible({ timeout: 15000 });
  });

  test("no-model guard CTA navigates to admin", async ({ page }) => {
    await page.goto("/threads");
    await page.waitForLoadState("domcontentloaded");

    // If the no-model guard is visible, clicking CTA should navigate to admin
    const guard = page.locator("text=No Model Configured").first();
    const isGuardVisible = await guard.isVisible().catch(() => false);

    if (isGuardVisible) {
      const ctaButton = page.locator("button:has-text('Configure Provider')").first();
      await ctaButton.click();
      await expect(page).toHaveURL(/\/admin/);
    } else {
      // No guard shown — model is configured, test passes trivially
      test.skip();
    }
  });
});
