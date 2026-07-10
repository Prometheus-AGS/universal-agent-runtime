import { createBdd } from 'playwright-bdd';
import { expect, test, openFreshThread, switchAgentViaUI } from '../support/world';
import { createTestAgent } from '../support/api';

const { Given, Then } = createBdd(test);

Given('a fresh conversation with an agent that has tools enabled', async ({ page }) => {
  const title = 'BDD Agent Tool-Enabled';
  await createTestAgent({ title, tools: ['native_echo'] });
  await openFreshThread(page);
  await switchAgentViaUI(page, title);
});

Then('a tool-call block for {string} is shown in the transcript', async ({ page }, toolName: string) => {
  await expect(page.getByText(toolName, { exact: true })).toBeVisible({ timeout: 10_000 });
});

Then('the tool-call result contains {string}', async ({ page }, needle: string) => {
  // tool-call-block.tsx's result panel is behind its expand toggle (aria-expanded).
  // exact:true — the thread sidebar's line-clamped conversation preview also
  // contains this substring, causing a strict-mode ambiguous match otherwise.
  const expandButton = page.locator('button[aria-expanded]').filter({ hasText: 'native_echo' }).first();
  await expandButton.click();
  await expect(page.getByText(needle, { exact: true })).toBeVisible({ timeout: 10_000 });
});
