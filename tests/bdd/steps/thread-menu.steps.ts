import { createBdd } from 'playwright-bdd';
import type { Locator, Page } from '@playwright/test';
import { expect, test, startNewConversation } from '../support/world';
import { renameActiveThread, stableThreadTitle, threadRow, threadRowTitle } from '../support/s1-ui';

const { Given, When, Then } = createBdd(test);

// Thread row actions menu (`frontend/src/components/layout/left-sidebar.tsx`): each row is a
// `div.group` holding the thread button (`aria-current="page"` when active) and a menu trigger
// labelled `Actions for <title>`, shown on hover. Rename opens a dialog with a `Thread name`
// field and Cancel / Save buttons; Delete opens an alert dialog whose description names the
// thread, with Cancel / Delete buttons. The list itself is `aria-label="Thread list"` and
// carries `aria-busy="true"` until the local store has hydrated.

const THREAD_LIST = '[aria-label="Thread list"]';

/** The title the active thread showed before it was renamed in this scenario, keyed by page. */
const previousTitles = new WeakMap<Page, string>();

function threadList(page: Page): Locator {
  return page.locator(THREAD_LIST);
}

/** The row of the thread open in the chat (its button carries `aria-current="page"`). */
function activeRow(page: Page): Locator {
  return threadList(page)
    .locator('div.group')
    .filter({ has: page.locator('button[aria-current="page"]') })
    .first();
}

/** The title text of a row (shared with the S1 harness). */
const rowTitle = threadRowTitle;

/**
 * Reads the active thread's title once it has stopped changing (a generated title can land
 * after the first message), and remembers it as the thread's previous title.
 */
async function rememberActiveTitle(page: Page): Promise<string> {
  const row = activeRow(page);
  await expect(row, 'the thread with the message is listed in the sidebar').toBeVisible({ timeout: 10_000 });
  const title = await stableThreadTitle(row);
  previousTitles.set(page, title);
  return title;
}

function previousTitleOf(page: Page): string {
  const title = previousTitles.get(page);
  if (title === undefined) throw new Error('no thread title was recorded before the rename in this scenario');
  return title;
}

/** Opens a row's actions menu and chooses one of its items. */
async function chooseRowAction(page: Page, row: Locator, title: string, item: 'Rename' | 'Delete'): Promise<void> {
  await expect(row, `the thread list shows a row for "${title}"`).toBeVisible({ timeout: 10_000 });
  await row.hover();
  await row.getByRole('button', { name: `Actions for ${title}`, exact: true }).click();
  await page.getByRole('menuitem', { name: item }).click();
}

function renameDialog(page: Page): Locator {
  return page.getByRole('dialog').filter({ hasText: 'Rename thread' });
}

function deleteConfirmation(page: Page): Locator {
  return page.getByRole('alertdialog').filter({ hasText: 'Delete thread?' });
}

/** Opens the Rename dialog of the active thread from its row menu and types a new name. */
async function startRenameOfActiveThread(page: Page, name: string): Promise<Locator> {
  const previous = await rememberActiveTitle(page);
  await chooseRowAction(page, activeRow(page), previous, 'Rename');
  const dialog = renameDialog(page);
  await expect(dialog, 'choosing Rename opens the rename dialog').toBeVisible({ timeout: 10_000 });
  await dialog.getByLabel('Thread name').fill(name);
  return dialog;
}

// ─── Given ──────────────────────────────────────────────────────────────────

// Setup only: names the active thread through the same Rename dialog the S1 harness uses,
// so a delete scenario can tell its two threads apart.
Given('this thread is named {string}', async ({ page }, name: string) => {
  await renameActiveThread(page, name);
});

Given('I start a new thread', async ({ page }) => {
  await startNewConversation(page);
});

// ─── When ───────────────────────────────────────────────────────────────────

When('I rename this thread to {string} through its row menu', async ({ page }, name: string) => {
  const dialog = await startRenameOfActiveThread(page, name);
  await dialog.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(dialog, 'saving closes the rename dialog').toBeHidden({ timeout: 10_000 });
});

When('I start renaming this thread to {string} and cancel', async ({ page }, name: string) => {
  const dialog = await startRenameOfActiveThread(page, name);
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click();
  await expect(dialog, 'cancelling closes the rename dialog').toBeHidden({ timeout: 10_000 });
});

When('I choose Delete in the row menu of {string}', async ({ page }, title: string) => {
  await chooseRowAction(page, threadRow(page, title).first(), title, 'Delete');
  await expect(deleteConfirmation(page), 'choosing Delete asks for confirmation').toBeVisible({ timeout: 10_000 });
});

When('I confirm the deletion', async ({ page }) => {
  const confirmation = deleteConfirmation(page);
  await confirmation.getByRole('button', { name: 'Delete', exact: true }).click();
  await expect(confirmation, 'confirming closes the confirmation').toBeHidden({ timeout: 10_000 });
});

When('I cancel the deletion', async ({ page }) => {
  const confirmation = deleteConfirmation(page);
  await confirmation.getByRole('button', { name: 'Cancel', exact: true }).click();
  await expect(confirmation, 'cancelling closes the confirmation').toBeHidden({ timeout: 10_000 });
});

When('I reload the page', async ({ page }) => {
  await page.reload();
  await page.locator('[aria-label="Select agent"]').waitFor({ state: 'visible', timeout: 15_000 });
  await expect(threadList(page), 'the thread list has loaded').toHaveAttribute('aria-busy', 'false', { timeout: 15_000 });
});

// ─── Then ───────────────────────────────────────────────────────────────────

Then('the renamed thread is titled {string} and no longer shows its previous title', async ({ page }, name: string) => {
  const previous = previousTitleOf(page);
  const title = rowTitle(activeRow(page));
  await expect(title, 'the renamed thread shows the new name').toHaveText(name, { timeout: 10_000 });
  if (previous !== name) await expect(title).not.toHaveText(previous);
});

Then('the thread list shows {string} exactly once', async ({ page }, title: string) => {
  const rows = threadRow(page, title);
  await expect(rows.first(), `the thread list shows "${title}"`).toBeVisible({ timeout: 15_000 });
  await expect(rows, `the thread list shows "${title}" once`).toHaveCount(1);
});

Then('the thread still shows its previous title', async ({ page }) => {
  const previous = previousTitleOf(page);
  await expect(rowTitle(activeRow(page)), 'the thread keeps its title').toHaveText(previous, { timeout: 10_000 });
});

Then('the thread list shows {string}', async ({ page }, title: string) => {
  await expect(threadRow(page, title).first(), `the thread list shows "${title}"`).toBeVisible({ timeout: 15_000 });
});

// Absence is only meaningful on a loaded list; the scenarios assert a remaining thread
// is shown before asserting a deleted one is not.
Then('the thread list does not show {string}', async ({ page }, title: string) => {
  await expect(threadList(page)).toHaveAttribute('aria-busy', 'false', { timeout: 15_000 });
  await expect(threadRow(page, title), `the thread list does not show "${title}"`).toHaveCount(0);
});

Then('the delete confirmation names {string}', async ({ page }, title: string) => {
  await expect(deleteConfirmation(page), 'the confirmation names the thread').toContainText(`"${title}"`);
});
