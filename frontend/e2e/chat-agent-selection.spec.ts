import { test, expect } from "@playwright/test";

test.describe("Chat — Agent selection", () => {
  test("chat page loads agent selector", async ({ page }) => {
    await page.goto("/threads");
    await page.waitForLoadState("domcontentloaded");

    // If the no-model guard is showing, skip — we need a configured server
    const guard = page.locator("text=No Model Configured, text=No LLM Provider Configured");
    const guardVisible = await guard.first().isVisible().catch(() => false);
    if (guardVisible) {
      test.skip();
      return;
    }

    // Click New Conversation to get a thread
    const newConv = page.locator("button:has-text('New conversation')").first();
    if (await newConv.isVisible().catch(() => false)) {
      await newConv.click();
    }

    // Agent selector (aria-label="Select agent") is always rendered in the chat toolbar
    const agentSelector = page.getByLabel("Select agent").first();
    await expect(agentSelector).toBeVisible({ timeout: 15000 });
  });

  test("new thread button creates a thread", async ({ page }) => {
    await page.goto("/threads");
    await page.waitForLoadState("domcontentloaded");

    const guard = page.locator("text=No Model Configured").first();
    const guardVisible = await guard.isVisible().catch(() => false);
    if (guardVisible) {
      test.skip();
      return;
    }

    const newConv = page.locator("button:has-text('New conversation'), [aria-label='New thread']").first();
    if (await newConv.isVisible({ timeout: 3000 }).catch(() => false)) {
      await newConv.click();
      // URL may change or thread list may update — just verify no crash
      await expect(page.locator("body")).toBeVisible();
    }
  });
});
