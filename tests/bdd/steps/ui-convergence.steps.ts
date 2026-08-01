import { createBdd } from 'playwright-bdd';
import type { Page } from '@playwright/test';
import { expect, test, waitForDbReady, openFreshThread } from '../support/world';

const { Given, When, Then } = createBdd(test);

const RAIL = 'nav[aria-label="Main (rail)"]';
const BOTTOM_BAR = 'nav[aria-label="Main (bottom bar)"]';

async function capture(page: Page, name: string): Promise<void> {
  await page.screenshot({ path: `test-results/evidence/${name}.png`, fullPage: false });
}

// ─── Given ──────────────────────────────────────────────────────────────────

Given('the app is open at {int} pixels wide', async ({ page }, width: number) => {
  await page.setViewportSize({ width, height: 900 });
  await waitForDbReady(page);
});

Given('the app is open', async ({ page }) => {
  await waitForDbReady(page);
});

Given('the stored theme is {string}', async ({ page }, theme: string) => {
  await page.addInitScript((t) => {
    try {
      localStorage.setItem('uar-theme', t);
    } catch {
      /* storage unavailable */
    }
  }, theme);
});

Given('a fresh conversation', async ({ page }) => {
  await openFreshThread(page);
});

// ─── When ───────────────────────────────────────────────────────────────────

When('the app loads', async ({ page }) => {
  await waitForDbReady(page);
});

When('I activate the theme toggle', async ({ page }) => {
  await page.locator('button[aria-label="Dark mode"]').click();
});

When('I tap the {string} destination in the bottom bar', async ({ page }, label: string) => {
  await page.locator(BOTTOM_BAR).getByRole('link', { name: label }).click();
});

// ─── Then: shell ────────────────────────────────────────────────────────────

Then('the rail shows destinations {string}, {string}, and {string}', async ({ page }, a: string, b: string, c: string) => {
  const rail = page.locator(RAIL);
  await expect(rail).toBeVisible();
  for (const label of [a, b, c]) {
    await expect(rail.getByRole('link', { name: label })).toBeVisible();
  }
});

Then('the active destination is {string}', async ({ page }, label: string) => {
  const active = page.locator(`${RAIL} a[aria-current="page"]`);
  await expect(active).toHaveText(new RegExp(label));
});

Then('the readiness lane reports {string} and {string}', async ({ page }, status: string, mode: string) => {
  const rail = page.locator(RAIL);
  await expect(rail.getByText(status, { exact: false })).toBeVisible();
  await expect(rail.getByText(mode, { exact: false })).toBeVisible();
});

Then('the bottom navigation bar shows destinations {string}, {string}, and {string}', async ({ page }, a: string, b: string, c: string) => {
  const bar = page.locator(BOTTOM_BAR);
  await expect(bar).toBeVisible();
  for (const label of [a, b, c]) {
    await expect(bar.getByRole('link', { name: label })).toBeVisible();
  }
});

Then('the rail is hidden', async ({ page }) => {
  await expect(page.locator(RAIL)).toBeHidden();
});

Then('the About page is shown', async ({ page }) => {
  await expect(page).toHaveURL(/\/about/);
  await expect(page.getByRole('heading', { name: /KnowMe/i }).first()).toBeVisible();
});

// ─── Then: theming ──────────────────────────────────────────────────────────

Then('the document theme is {string}', async ({ page }, theme: string) => {
  await expect
    .poll(() => page.evaluate(() => document.documentElement.className), { timeout: 10_000 })
    .toContain(theme);
});

Then('the toggle reports {string}', async ({ page }, label: string) => {
  await expect(page.locator(`button[aria-label="${label}"]`)).toBeVisible();
});

Then('no element in the chat surface has a visible border', async ({ page }) => {
  // Flat 2.0 tolerates borders whose color matches the element's own fill
  // (surface-filled controls like border-input are invisible in practice).
  // A border is "visible" only when it contrasts with what it sits on.
  const offenders = await page.evaluate(() => {
    const parse = (c: string) => c.match(/\d+(\.\d+)?/g)?.map(Number) ?? [0, 0, 0, 0];
    const differs = (a: string, b: string) => {
      const pa = parse(a), pb = parse(b);
      const drift = Math.abs(pa[0] - pb[0]) + Math.abs(pa[1] - pb[1]) + Math.abs(pa[2] - pb[2]);
      const alpha = pa[3] !== undefined ? pa[3] : 1;
      return alpha > 0.05 && drift > 12;
    };
    return [...document.querySelectorAll('main *')]
      .filter((el) => {
        const cs = getComputedStyle(el);
        const bg = cs.backgroundColor;
        const sides: Array<[string, string]> = [
          [cs.borderTopWidth, cs.borderTopColor],
          [cs.borderRightWidth, cs.borderRightColor],
          [cs.borderBottomWidth, cs.borderBottomColor],
          [cs.borderLeftWidth, cs.borderLeftColor],
        ];
        return sides.some(([w, color]) => parseFloat(w) > 0 && differs(color, bg));
      })
      .map((el) => `${el.tagName}.${String((el as HTMLElement).className).slice(0, 60)}`);
  });
  expect(offenders, `visible borders found: ${offenders.slice(0, 5).join(' | ')}`).toEqual([]);
});

Then('no element in the chat surface has a layout shadow', async ({ page }) => {
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  const offenders = await page.evaluate(() => {
    const els = [...document.querySelectorAll('main *')];
    return els
      .filter((el) => {
        const shadow = getComputedStyle(el).boxShadow;
        return shadow !== 'none' && !shadow.includes('rgba(0, 0, 0, 0) 0px 0px 0px 0px');
      })
      .map((el) => `${el.tagName}.${String((el as HTMLElement).className).slice(0, 60)}`);
  });
  expect(offenders, `layout shadows found: ${offenders.slice(0, 5).join(' | ')}`).toEqual([]);
});

// ─── Then: chat anatomy ─────────────────────────────────────────────────────

Then('the composer shell has no visible border and no shadow', async ({ page }) => {
  const input = page.locator('[aria-label="Message input"]');
  await expect(input).toBeVisible();
  // The AttachmentDropzone is the input's direct parent — the filled composer shell.
  const shell = input.locator('xpath=..');
  const metrics = await shell.evaluate((el) => {
    const cs = getComputedStyle(el);
    return {
      borderWidths: [cs.borderTopWidth, cs.borderRightWidth, cs.borderBottomWidth, cs.borderLeftWidth],
      borderColors: [cs.borderTopColor, cs.borderRightColor, cs.borderBottomColor, cs.borderLeftColor],
      boxShadow: cs.boxShadow,
      bg: cs.backgroundColor,
      classes: String((el as HTMLElement).className),
    };
  });
  expect(metrics.classes, 'composer shell is not the rounded filled dropzone').toContain('rounded');
  const hasVisibleBorder = metrics.borderWidths.some(
    (w, i) => parseFloat(w) > 0 && metrics.borderColors[i] !== metrics.bg && metrics.borderColors[i] !== 'rgba(0, 0, 0, 0)' && metrics.borderColors[i] !== 'transparent',
  );
  expect(hasVisibleBorder, `composer shell draws a visible border: ${JSON.stringify(metrics)}`).toBe(false);
  expect(metrics.boxShadow === 'none' || metrics.boxShadow.includes('rgba(0, 0, 0, 0) 0px 0px 0px 0px'), 'composer shell has a shadow').toBe(true);
});

Then('the user bubble is trailing-aligned on an ember-tinted surface', async ({ page }) => {
  const userRoot = page.locator('[data-role="user"]').last();
  await expect(userRoot).toBeVisible({ timeout: 15_000 });
  const bubble = userRoot.locator('div[class*="bg-ember-soft"]').first();
  await expect(bubble).toBeVisible();
  await expect(bubble).toHaveClass(/rounded-ee-md/);
  const box = await bubble.boundingBox();
  const viewport = page.viewportSize();
  expect(box, 'bubble has no layout box').toBeTruthy();
  expect(box!.x + box!.width / 2, 'user bubble is not on the trailing (right) half').toBeGreaterThan(viewport!.width / 2);
});

// ─── Then: evidence ─────────────────────────────────────────────────────────

Then('I capture a screenshot named {string}', async ({ page }, name: string) => {
  await capture(page, name);
});
