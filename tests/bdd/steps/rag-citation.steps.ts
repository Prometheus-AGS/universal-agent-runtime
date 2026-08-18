import { createBdd } from 'playwright-bdd';
import { expect, test } from '../support/world';

const { Then } = createBdd(test);

// `MessageCitations` (frontend/src/components/citations/citation-hover-panel.tsx)
// renders a `[aria-label="Sources"]` row of numbered badges only when the
// message carries a non-empty `rag-citations` content block — itself only
// populated when the backend emits `NormalizedEvent::RagCitations` (built
// from `CitationStream`) after a non-empty RAG retrieval.

Then('a RAG citation source badge is shown in the transcript', async ({ page }) => {
  const sources = page.locator('[aria-label="Sources"]').last();
  await expect(sources).toBeVisible({ timeout: 15_000 });
  await expect(sources.getByRole('button').first()).toBeVisible();
});

Then('no RAG citation source badge is shown in the transcript', async ({ page }) => {
  await expect(page.locator('[aria-label="Sources"]')).toHaveCount(0);
});

Then('hovering the first citation badge reveals its source document', async ({ page }) => {
  const badge = page.locator('[aria-label="Sources"]').last().getByRole('button').first();
  await badge.hover();
  // The ingested BDD fixture is always uploaded as `bdd-fixture.txt`
  // (see `createKnowledgeBaseWithDocument` in support/api.ts), and the
  // backend resolves that filename as the citation's `document_name`.
  const hoverCard = page.locator('[data-slot="hover-card-content"]');
  await expect(hoverCard.getByText('bdd-fixture.txt', { exact: true })).toBeVisible({ timeout: 5_000 });
});
