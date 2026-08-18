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
}

const evidence = new WeakMap<Page, LocalEvidence>();

type RuntimeReplayWindow = Window & {
  __uarRuntimeReplay?: { reset: () => void; replayAll: () => void };
  __bddSyncProbe?: { opens: number; source: EventSource };
};

async function replayRuntime(page: Page, reset: boolean): Promise<void> {
  await page.waitForFunction(() => Boolean((window as RuntimeReplayWindow).__uarRuntimeReplay));
  await page.evaluate((shouldReset) => {
    const replay = (window as RuntimeReplayWindow).__uarRuntimeReplay;
    if (!replay) throw new Error('runtime replay helper not installed');
    if (shouldReset) replay.reset();
    replay.replayAll();
  }, reset);
}

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

Given('the runtime cockpit has one known replayed run', async ({ page }) => {
  await page.goto('/admin/runtime');
  await expect(page.getByTestId('admin-section-runtime')).toBeVisible({ timeout: 15_000 });
  await page.evaluate(() => {
    const source = new EventSource('/api/uar/sync/stream');
    const probe = { opens: 0, source };
    source.onopen = () => { probe.opens += 1; };
    (window as RuntimeReplayWindow).__bddSyncProbe = probe;
  });
  await page.waitForFunction(() => (window as RuntimeReplayWindow).__bddSyncProbe?.opens === 1);
  await replayRuntime(page, true);
  await expect(page.getByText('Live Replay Run')).toHaveCount(1);
});

When('the embedded sync stream disconnects and reconnects', async ({ context, page }) => {
  await context.setOffline(true);
  await expect(page.getByRole('alert').getByText(/You are offline/)).toBeVisible();
  await context.setOffline(false);
  const reconnect = page.waitForRequest(
    (request) => request.url().includes('/api/uar/sync/stream'),
    { timeout: 15_000 },
  );
  await page.reload();
  await reconnect;
  await expect(page.getByTestId('admin-section-runtime')).toBeVisible({ timeout: 15_000 });
  await replayRuntime(page, false);
  evidence.set(page, { ...evidence.get(page), reconnected: true });
});

Then('the restored cockpit still contains exactly one known run', async ({ page }) => {
  expect(evidence.get(page)?.reconnected).toBe(true);
  await expect(page.getByText('Live Replay Run')).toHaveCount(1);
  await expect(page.getByText('Replay tool execution')).toBeVisible();
});
