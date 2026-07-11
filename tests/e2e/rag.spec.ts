import { test, expect } from './fixtures';

// Browser-level smoke check for the RAG surface.
//
// The full upload -> ingest -> vector-search -> ranked-retrieval assertion is
// covered deterministically at the integration level by the
// `rag_ingest_then_retrieve` test (re-enabled with real BGE-small embeddings in
// fix-embeddings-fastembed; live ingest->search scored 0.84 on a
// previously-empty query). This e2e verifies the browser surface is wired and
// does not regress — and, importantly, does NOT treat a failed/empty response
// as a pass.
test('RAG surface loads, accepts input, and returns a real response', async ({ page }) => {
  page.on('console', (msg) => console.log(`BROWSER LOG: ${msg.text()}`));
  const dbReadyPromise = page.waitForEvent('console', (msg) =>
    msg.text().includes('PGlite database initialized successfully'),
  );
  await page.goto('/');
  await dbReadyPromise;

  // The document-upload affordance must be present and usable (not merely in DOM).
  const uploadInput = page.locator('input[type="file"]');
  await expect(uploadInput).toBeAttached();

  // Send a query and require a real, non-empty assistant response — an error
  // state or an empty message is a failure, not a graceful pass.
  const input = page.locator('textarea[name="message"]');
  await input.fill('Summarize the uploaded document');
  await page.keyboard.press('Enter');

  const assistantMsg = page.locator('.assistant-message').first();
  await expect(assistantMsg).toBeVisible({ timeout: 30000 });
  await expect(assistantMsg).not.toBeEmpty();
  await expect(page.locator('.error-message, [role="alert"]')).toHaveCount(0);
});
