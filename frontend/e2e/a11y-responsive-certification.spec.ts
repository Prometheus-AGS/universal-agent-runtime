import { createRequire } from "node:module";
import { expect, test } from "@chromatic-com/playwright";
import type { AxeResults, Result as AxeViolation } from "axe-core";
import type { Locator, Page } from "@playwright/test";

const require = createRequire(import.meta.url);
const axePath = require.resolve("axe-core/axe.min.js");

const widths = [320, 768, 1024, 1440] as const;
const themes = ["light", "dark"] as const;

type CertifiedTheme = (typeof themes)[number] | "high-contrast";

function formatViolations(violations: AxeViolation[]): string {
  return violations
    .map((violation) => {
      const nodes = violation.nodes
        .map((node) => `${node.target.join(" ")}: ${node.failureSummary ?? node.html}`)
        .join("\n    ");
      return `${violation.id} [${violation.impact ?? "unknown"}] ${violation.help}\n    ${nodes}`;
    })
    .join("\n\n");
}

async function installDeterministicRoutes(page: Page) {
  await page.route(/\/api\/agents(?:\?.*)?$/, async (route) => {
    await route.fulfill({ contentType: "application/json", body: "[]" });
  });
  await page.route(/\/api\/uar\/providers(?:\?.*)?$/, async (route) => {
    await route.fulfill({ contentType: "application/json", body: "[]" });
  });
  await page.route(/\/api\/uar\/settings\/(?:types|[a-z-]+)(?:\?.*)?$/, async (route) => {
    await route.fulfill({ contentType: "application/json", body: "[]" });
  });
  await page.route(/\/healthz(?:\?.*)?$/, async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ status: "ok", version: "certification" }),
    });
  });
  await page.route(/\/api\/uar\/resolve-model(?:\?.*)?$/, async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ ok: true, provider_id: "local", model_id: "local/certification" }),
    });
  });
  await page.route(/\/api\/uar\/providers\/health(?:\?.*)?$/, async (route) => {
    await route.fulfill({ contentType: "application/json", body: JSON.stringify({ providers: {} }) });
  });
  await page.route(/\/api\/uar\/a2ui\/schemas(?:\?.*)?$/, async (route) => {
    await route.fulfill({ contentType: "application/json", body: "[]" });
  });
  await page.route(/\/api\/config\/persistence(?:\?.*)?$/, async (route) => {
    await route.fulfill({ status: 204 });
  });
}

async function openSurface(
  page: Page,
  path: string,
  theme: CertifiedTheme,
  width: number,
) {
  await installDeterministicRoutes(page);
  await page.setViewportSize({ width, height: 900 });
  await page.addInitScript((storedTheme) => {
    localStorage.setItem("uar-theme", storedTheme);
  }, theme);
  await page.emulateMedia({ colorScheme: theme === "light" ? "light" : "dark" });
  await page.goto(path, { waitUntil: "domcontentloaded" });
  await expect(page.locator("#shell-main-content")).toBeVisible();
  await expect(page.locator("html")).toHaveClass(new RegExp(`(?:^|\\s)${theme}(?:\\s|$)`));
}

async function expectNoAxeViolations(page: Page) {
  await page.addScriptTag({ path: axePath });
  const results = await page.evaluate(async () => {
    const axe = (window as typeof window & {
      axe: { run: (context: Document, options: object) => Promise<AxeResults> };
    }).axe;

    return axe.run(document, {
      runOnly: {
        type: "tag",
        values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"],
      },
      resultTypes: ["violations"],
    });
  });

  expect(results.violations, formatViolations(results.violations)).toEqual([]);
}

async function expectNoGlobalOverflow(page: Page) {
  const overflow = await page.evaluate(() => ({
    documentWidth: document.documentElement.scrollWidth,
    viewportWidth: window.innerWidth,
  }));
  expect(overflow.documentWidth).toBeLessThanOrEqual(overflow.viewportWidth);
}

async function expectNoOverlap(first: Locator, second: Locator) {
  const [a, b] = await Promise.all([first.boundingBox(), second.boundingBox()]);
  expect(a).not.toBeNull();
  expect(b).not.toBeNull();
  if (!a || !b) return;

  const xOverlap = Math.max(0, Math.min(a.x + a.width, b.x + b.width) - Math.max(a.x, b.x));
  const yOverlap = Math.max(0, Math.min(a.y + a.height, b.y + b.height) - Math.max(a.y, b.y));
  expect(xOverlap * yOverlap).toBe(0);
}

async function expectStandaloneTargetsAtLeast24px(page: Page) {
  const undersized = await page.locator([
    "a[href]",
    "button",
    "input:not([type=hidden])",
    "select",
    "textarea",
    "summary",
    "[role=button]",
    "[role=tab]",
    "[role=menuitem]",
    "[role=checkbox]",
    "[role=radio]",
    "[role=switch]",
    "[role=combobox]",
  ].join(",")).evaluateAll((elements) => elements.flatMap((element) => {
    const htmlElement = element as HTMLElement;
    const style = getComputedStyle(htmlElement);
    const rect = htmlElement.getBoundingClientRect();
    const visible = style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
    if (!visible || (rect.width >= 24 && rect.height >= 24)) return [];
    return [{
      name: htmlElement.getAttribute("aria-label") ?? htmlElement.textContent?.trim().slice(0, 80) ?? htmlElement.tagName,
      width: rect.width,
      height: rect.height,
      html: htmlElement.outerHTML.slice(0, 240),
    }];
  }));

  expect(undersized, JSON.stringify(undersized, null, 2)).toEqual([]);
}

async function tabUntilFocused(page: Page, target: Locator, key: "Tab" | "Shift+Tab" = "Tab") {
  for (let step = 0; step < 80; step += 1) {
    await page.keyboard.press(key);
    if (await target.evaluate((element) => element === document.activeElement)) return;
  }
  throw new Error(`Keyboard traversal did not reach ${await target.getAttribute("aria-label") ?? "target"}`);
}

async function expectThreePixelFocusWithContrast(target: Locator) {
  await target.focus();
  await target.page().waitForTimeout(250);
  const focus = await target.evaluate((element) => {
    const parseRgb = (value: string) => {
      const channels = value.match(/[\d.]+/g)?.slice(0, 4).map(Number) ?? [];
      return {
        red: channels[0] ?? 0,
        green: channels[1] ?? 0,
        blue: channels[2] ?? 0,
        alpha: channels[3] ?? 1,
      };
    };
    const luminance = ({ red, green, blue }: ReturnType<typeof parseRgb>) => {
      const linear = [red, green, blue].map((channel) => {
        const value = channel / 255;
        return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
      });
      return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
    };
    const parseHex = (value: string) => {
      const normalized = value.trim().replace("#", "");
      return {
        red: Number.parseInt(normalized.slice(0, 2), 16),
        green: Number.parseInt(normalized.slice(2, 4), 16),
        blue: Number.parseInt(normalized.slice(4, 6), 16),
        alpha: 1,
      };
    };
    const contrast = (first: ReturnType<typeof parseRgb>, second: ReturnType<typeof parseRgb>) => {
      const light = Math.max(luminance(first), luminance(second));
      const dark = Math.min(luminance(first), luminance(second));
      return (light + 0.05) / (dark + 0.05);
    };
    const style = getComputedStyle(element);
    const shadows = Array.from(
      style.boxShadow.matchAll(/((?:rgba?|oklab)\([^)]+\))\s+0px\s+0px\s+0px\s+([\d.]+)px/g),
      (match) => ({ color: match[1], spread: Number(match[2]) }),
    ).filter((shadow) => shadow.spread > 0)
      .sort((first, second) => second.spread - first.spread);
    const indicatorShadow = shadows[0];
    const offsetShadow = shadows[1];
    let ancestor: Element | null = element.parentElement;
    let adjacent = "rgb(255, 255, 255)";
    while (ancestor) {
      const candidate = getComputedStyle(ancestor).backgroundColor;
      if (parseRgb(candidate).alpha > 0.01) {
        adjacent = candidate;
        break;
      }
      ancestor = ancestor.parentElement;
    }
    const indicator = getComputedStyle(document.documentElement).getPropertyValue("--color-focus-ring").trim();
    return {
      boxShadow: style.boxShadow,
      indicator,
      thickness: (indicatorShadow?.spread ?? 0) - (offsetShadow?.spread ?? 0),
      adjacent,
      contrast: indicator ? contrast(parseHex(indicator), parseRgb(adjacent)) : 0,
    };
  });

  expect(focus.boxShadow).not.toBe("none");
  expect(focus.indicator, focus.boxShadow).not.toBe("");
  expect(focus.thickness, focus.boxShadow).toBe(3);
  expect(focus.contrast, `${focus.indicator} against ${focus.adjacent}`).toBeGreaterThanOrEqual(3);
}

test.describe("WCAG and responsive certification", () => {
  for (const theme of themes) {
    for (const width of widths) {
      test(`${theme} runtime surface passes at ${width}px`, async ({ page }) => {
        await openSurface(page, "/admin/runtime", theme, width);
        await expect(page.getByRole("heading", { name: "Live Runs" })).toBeVisible();
        await expect(page.locator("main")).toHaveCount(1);

        const main = page.locator("#shell-main-content");
        if (width <= 900) {
          const compactNav = page.getByRole("navigation", { name: "Compact navigation" });
          await expect(compactNav).toBeVisible();
          await expect(page.getByRole("navigation", { name: "Primary navigation" })).not.toBeVisible();
          await expectNoOverlap(main, compactNav);
        } else {
          const primaryNav = page.getByRole("navigation", { name: "Primary navigation" });
          await expect(primaryNav).toBeVisible();
          await expect(page.getByRole("navigation", { name: "Compact navigation" })).not.toBeVisible();
          await expectNoOverlap(primaryNav, main);
        }

        await expectNoGlobalOverflow(page);
        await expectStandaloneTargetsAtLeast24px(page);
        await expectNoAxeViolations(page);
      });
    }
  }

  for (const theme of themes) {
    test(`${theme} chat and settings retain one application main landmark`, async ({ page }) => {
      for (const path of ["/threads", "/admin/settings"]) {
        await openSurface(page, path, theme, 1440);
        await expect(page.locator("main")).toHaveCount(1);
        await expectNoAxeViolations(page);
      }
    });
  }

  test("high-contrast runtime surface passes the accessibility gate", async ({ page }) => {
    await openSurface(page, "/admin/runtime", "high-contrast", 1440);
    await expectNoAxeViolations(page);
  });

  test("desktop skip link and command palette are keyboard operable", async ({ page }) => {
    await openSurface(page, "/admin/runtime", "dark", 1440);

    await page.keyboard.press("Tab");
    const skipLink = page.getByRole("link", { name: "Skip to content" });
    await expect(skipLink).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(page.locator("#shell-main-content")).toBeFocused();

    const commandTrigger = page.getByRole("button", { name: "Open command palette" });
    await tabUntilFocused(page, commandTrigger);
    await page.keyboard.press("Enter");
    await expect(page.getByRole("dialog", { name: "Command palette" })).toBeVisible();
    const search = page.getByRole("combobox", { name: "Search commands" });
    await expect(search).toBeFocused();
    await search.fill("Runtime settings");
    await page.keyboard.press("Enter");
    await expect(page).toHaveURL(/\/admin\/settings$/);
    await expect(page.getByRole("dialog", { name: "Command palette" })).not.toBeVisible();
    await expect(commandTrigger).toBeFocused();

    await page.keyboard.press("Enter");
    await expect(page.getByRole("dialog", { name: "Command palette" })).toBeVisible();
    await page.keyboard.press("Shift+Tab");
    await expect(page.getByRole("dialog", { name: "Command palette" }).locator(":focus")).toHaveCount(1);
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog", { name: "Command palette" })).not.toBeVisible();
    await expect(commandTrigger).toBeFocused();
  });

  test("compact configure dialog is keyboard operable and restores focus", async ({ page }) => {
    await openSurface(page, "/admin/runtime", "dark", 320);
    const configureTrigger = page.getByRole("button", { name: "Configure" });
    await page.locator("body").focus();
    await tabUntilFocused(page, configureTrigger);
    await page.keyboard.press("Enter");
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();
    const actions = dialog.locator("button:not([disabled]), a[href]");
    const actionCount = await actions.count();
    const visited = new Set<string>();
    for (let step = 0; step < actionCount + 1; step += 1) {
      const activeLabel = await page.evaluate(() => {
        const active = document.activeElement as HTMLElement | null;
        return active?.getAttribute("aria-label") ?? active?.textContent?.trim() ?? "";
      });
      if (activeLabel) visited.add(activeLabel);
      await page.keyboard.press("Tab");
      await expect(dialog.locator(":focus")).toHaveCount(1);
    }
    expect(visited.size).toBe(actionCount);
    await page.keyboard.press("Shift+Tab");
    await expect(dialog.locator(":focus")).toHaveCount(1);
    await page.keyboard.press("Escape");
    await expect(dialog).not.toBeVisible();
    await expect(configureTrigger).toBeFocused();
  });

  for (const theme of themes) {
    test(`${theme} focus presentation is 3px with 3:1 adjacent contrast`, async ({ page }) => {
      await openSurface(page, "/admin/runtime", theme, 1440);
      const commandTrigger = page.getByRole("button", { name: "Open command palette" });
      await expectThreePixelFocusWithContrast(page.getByRole("link", { name: "Skip to content" }));
      await expectThreePixelFocusWithContrast(commandTrigger);
      await expectThreePixelFocusWithContrast(page.getByRole("button", { name: theme === "dark" ? "Dark mode" : "Light mode" }));

      await commandTrigger.focus();
      await page.keyboard.press("Enter");
      await expectThreePixelFocusWithContrast(page.getByRole("button", { name: "Close command palette" }));
      await page.keyboard.press("Escape");

      await openSurface(page, "/threads", theme, 1440);
      await expectThreePixelFocusWithContrast(page.getByRole("button", { name: "New conversation" }));
    });
  }

  test("status is textual and reduced-motion collapses non-essential motion", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await openSurface(page, "/admin/runtime", "dark", 1440);

    const readiness = page.getByRole("navigation", { name: "Primary navigation" });
    const readyText = readiness.getByText("Ready", { exact: true });
    await expect(readyText).toBeVisible();
    await expect(readiness.getByText(/Embedded · local/)).toBeVisible();
    const readinessDot = readyText.locator("xpath=../preceding-sibling::span");
    await expect(readinessDot).toHaveAttribute("aria-hidden", "true");

    const transitionDuration = await page.getByRole("navigation", { name: "Primary navigation" })
      .evaluate((element) => getComputedStyle(element).transitionDuration);
    expect(transitionDuration).toBe("0.001s");
  });
});
