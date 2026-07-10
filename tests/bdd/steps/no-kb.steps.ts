import { createBdd } from 'playwright-bdd';
import { expect, test, openFreshThread, switchAgentViaUI } from '../support/world';
import { createTestAgent } from '../support/api';

const { Given, Then } = createBdd(test);

Given('a fresh conversation with an agent that has no knowledge bases attached', async ({ page }) => {
  const title = 'BDD Agent No-KB';
  await createTestAgent({ title });
  await openFreshThread(page);
  await switchAgentViaUI(page, title);
});

Then('no knowledge-base retrieval indicator is shown in the transcript', async ({ page }) => {
  // capability-toggles.tsx's "Knowledge" toggle (`aria-label="Toggle Knowledge"`)
  // only shows a count badge (`CountBadge`, a `\d+` span) when at least one KB
  // is attached — for a KB-less agent it renders with no numeric badge at all.
  const knowledgeToggle = page.locator('[aria-label="Toggle Knowledge"]');
  await expect(knowledgeToggle).toBeVisible();
  await expect(knowledgeToggle.getByText(/^\d+$/)).toHaveCount(0);
});
