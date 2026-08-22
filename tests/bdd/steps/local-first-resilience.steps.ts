import { createBdd } from 'playwright-bdd';
import type { Page } from '@playwright/test';
import {
  expect,
  openFreshThread,
  sendMessageAndWait,
  test,
  waitForDbReady,
} from '../support/world';

const { Given, When, Then } = createBdd(test);

interface LocalEvidence {
  offline?: boolean;
  online?: boolean;
  retained?: boolean;
  freshIsolated?: boolean;
  reconnected?: boolean;
  syncEntityId?: string;
  syncInitialName?: string;
  syncRecoveredName?: string;
  syncSourceCount?: number;
}

const evidence = new WeakMap<Page, LocalEvidence>();

type EmbeddedSseWindow = Window & {
  __bddEmbeddedSse?: { sources: EventSource[] };
};

Given('the threads screen is ready', async ({ page }) => {
  await waitForDbReady(page);
});

When('the browser goes offline and returns online', async ({ context, page }) => {
  await context.setOffline(true);
  await expect(page.getByRole('alert').getByText(/You are offline/)).toBeVisible();
  evidence.set(page, { ...evidence.get(page), offline: true });
  await context.setOffline(false);
  await expect(page.getByRole('alert')).not.toBeVisible();
  evidence.set(page, { ...evidence.get(page), online: true });
});

Then('the offline banner appears and then clears', async ({ page }) => {
  expect(evidence.get(page)).toMatchObject({ offline: true, online: true });
});

Given('a completed deterministic chat is stored locally', async ({ page }) => {
  await openFreshThread(page);
  await sendMessageAndWait(page, 'What is 2 plus 2?');
  await expect(page.getByText('2 plus 2 is 4.', { exact: false }).last()).toBeVisible();
  await expect(page.getByRole('button', { name: /What is 2 plus 2\?/ }).first()).toBeVisible();
  await page.waitForTimeout(500);
});

When('I reload it and open a fresh browser context', async ({ browser, page }) => {
  await page.reload();
  await page.locator('[aria-label="Select agent"]').waitFor({ state: 'visible', timeout: 15_000 });
  await page.getByRole('button', { name: /What is 2 plus 2\?/ }).first().click();
  await expect(page.getByText('2 plus 2 is 4.', { exact: false }).last()).toBeVisible();
  evidence.set(page, { ...evidence.get(page), retained: true });

  const freshContext = await browser.newContext();
  const freshPage = await freshContext.newPage();
  try {
    await freshPage.goto('/threads');
    await freshPage.locator('[aria-label="Select agent"]').waitFor({ state: 'visible', timeout: 15_000 });
    await expect(freshPage.getByRole('button', { name: /What is 2 plus 2\?/ })).toHaveCount(0);
    evidence.set(page, { ...evidence.get(page), freshIsolated: true });
  } finally {
    await freshContext.close();
  }
});

Then('the original context retains the answer and the fresh context does not inherit it', async ({ page }) => {
  expect(evidence.get(page)).toMatchObject({ retained: true, freshIsolated: true });
  await expect(page.getByText('2 plus 2 is 4.', { exact: false }).last()).toBeVisible();
});

Given('a known knowledge base is visible through the registered embedded stream', async ({ page }) => {
  await page.addInitScript(() => {
    const NativeEventSource = window.EventSource;
    const state = { sources: [] as EventSource[] };
    const ObservedEventSource = new Proxy(NativeEventSource, {
      construct(target, args) {
        const source = Reflect.construct(target, args) as EventSource;
        if (String(args[0]).includes('/api/uar/sync/stream')) {
          state.sources.push(source);
        }
        return source;
      },
    });
    Object.defineProperty(window, 'EventSource', {
      configurable: true,
      value: ObservedEventSource,
      writable: true,
    });
    (window as EmbeddedSseWindow).__bddEmbeddedSse = state;
  });

  await page.goto('/admin/knowledge');
  await expect(page.getByRole('heading', { name: 'Knowledge Bases' })).toBeVisible();
  await page.waitForFunction(() => {
    const sources = (window as EmbeddedSseWindow).__bddEmbeddedSse?.sources ?? [];
    return sources.length === 1 && sources[0]?.readyState === 1;
  });

  const baseName = `BDD SSE KB ${Date.now()}`;
  await page.getByRole('button', { name: 'New', exact: true }).click();
  await page.getByLabel('Name', { exact: true }).fill(baseName);
  await page.getByLabel('Description', { exact: true }).fill('Embedded SSE fixture');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  const baseCard = page.getByTestId(/^knowledge-base-/).filter({ hasText: baseName });
  await expect(baseCard).toBeVisible();
  const testId = await baseCard.getAttribute('data-testid');
  if (!testId?.startsWith('knowledge-base-')) {
    throw new Error(`knowledge base test id missing: ${testId ?? '<none>'}`);
  }

  const entityId = testId.slice('knowledge-base-'.length);
  const initialName = `${baseName} initial`;
  await page.evaluate(({ entityId, initialName }) => {
    const source = (window as EmbeddedSseWindow).__bddEmbeddedSse?.sources.at(-1);
    if (!source) throw new Error('registered embedded EventSource not captured');
    source.dispatchEvent(new MessageEvent('entity.change', {
      data: JSON.stringify({
        table: 'knowledge_bases',
        action: 'update',
        id: entityId,
        record: {
          id: entityId,
          name: initialName,
          description: 'Embedded SSE fixture',
          document_count: 0,
        },
      }),
    }));
  }, { entityId, initialName });
  await expect(page.getByTestId(testId)).toContainText(initialName);
  evidence.set(page, {
    ...evidence.get(page),
    syncEntityId: entityId,
    syncInitialName: initialName,
    syncRecoveredName: `${baseName} recovered`,
    syncSourceCount: 1,
  });
});

When('the registered embedded sync stream reports an error and reconnects', async ({ page }) => {
  const current = evidence.get(page);
  if (!current?.syncEntityId || !current.syncRecoveredName || !current.syncSourceCount) {
    throw new Error('embedded SSE evidence was not initialized');
  }
  const reconnect = page.waitForRequest(
    (request) => request.url().includes('/api/uar/sync/stream'),
    { timeout: 15_000 },
  );
  await page.evaluate(() => {
    const source = (window as EmbeddedSseWindow).__bddEmbeddedSse?.sources.at(-1);
    if (!source) throw new Error('registered embedded EventSource not captured');
    source.dispatchEvent(new Event('error'));
  });
  const update = await page.evaluate(async ({ entityId, recoveredName, previousCount }) => {
    const response = await fetch(`/api/knowledge/${encodeURIComponent(entityId)}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: recoveredName }),
    });
    const body = await response.json() as { name?: string };
    const sourceCount = (window as EmbeddedSseWindow).__bddEmbeddedSse?.sources.length ?? 0;
    return { status: response.status, name: body.name, sourceCount, previousCount };
  }, {
    entityId: current.syncEntityId,
    recoveredName: current.syncRecoveredName,
    previousCount: current.syncSourceCount,
  });
  expect(update).toEqual({
    status: 200,
    name: current.syncRecoveredName,
    sourceCount: current.syncSourceCount,
    previousCount: current.syncSourceCount,
  });
  await reconnect;
  await page.waitForFunction((previousCount) => {
    const sources = (window as EmbeddedSseWindow).__bddEmbeddedSse?.sources ?? [];
    return sources.length === previousCount + 1 && sources.at(-1)?.readyState === 1;
  }, current.syncSourceCount, { timeout: 15_000 });
  evidence.set(page, { ...evidence.get(page), reconnected: true });
});

Then('the knowledge screen contains exactly one recovered knowledge base', async ({ page }) => {
  const current = evidence.get(page);
  expect(current?.reconnected).toBe(true);
  if (!current?.syncEntityId || !current.syncRecoveredName) {
    throw new Error('embedded SSE recovery evidence is incomplete');
  }
  await expect(page.getByTestId(`knowledge-base-${current.syncEntityId}`)).toContainText(
    current.syncRecoveredName,
  );
  await expect(page.getByText(current.syncRecoveredName, { exact: true })).toHaveCount(1);
});
