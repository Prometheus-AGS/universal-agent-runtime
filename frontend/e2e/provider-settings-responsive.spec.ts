import { expect, test } from "@playwright/test";
import type { Locator, Page } from "@playwright/test";

const providerName = "Long-Context Deterministic Provider";
const providerKey = "provider.responsive-fixture";
const overflowTolerance = 1;

const providerSetting = {
  id: "setting-provider-responsive-fixture",
  settings_type_id: "settings-type-provider",
  key: providerKey,
  name: providerName,
  data: {
    display_name: providerName,
    base_url:
      "https://gateway.responsive-fixture.example.test/v1/organizations/long-context-workloads",
    api_key: "************",
    protocol: "responses",
    enabled: true,
    default_model: "responsive-model",
    models: [
      {
        id: "responsive-model",
        display_name: "Responsive Model With A Representative Long Name",
        enabled: true,
      },
    ],
  },
  created_at: "2026-08-26T00:00:00Z",
  meta: { source: "Api", is_drift: false },
};

async function installDeterministicRoutes(page: Page) {
  const durableSettingsWrites: Array<{ method: string; path: string }> = [];

  await page.route(/\/api\/uar\/settings(?:\/[^?]*)?(?:\?.*)?$/, async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    if (request.method() !== "GET") {
      durableSettingsWrites.push({ method: request.method(), path: url.pathname });
      await route.fulfill({
        status: 409,
        contentType: "application/json",
        body: JSON.stringify({ error: "Unexpected durable settings write" }),
      });
      return;
    }

    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify(url.pathname.endsWith("/providers") ? [providerSetting] : []),
    });
  });
  await page.route(/\/api\/agents(?:\?.*)?$/, async (route) => {
    await route.fulfill({ contentType: "application/json", body: "[]" });
  });
  await page.route(/\/api\/uar\/providers(?:\?.*)?$/, async (route) => {
    await route.fulfill({ contentType: "application/json", body: "[]" });
  });
  await page.route(/\/healthz(?:\?.*)?$/, async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ status: "ok", version: "responsive-fixture" }),
    });
  });
  await page.route(/\/api\/uar\/resolve-model(?:\?.*)?$/, async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({
        ok: true,
        provider_id: "responsive-fixture",
        model_id: "responsive-model",
      }),
    });
  });
  await page.route(/\/api\/uar\/providers\/health(?:\?.*)?$/, async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ providers: {} }),
    });
  });
  await page.route(/\/api\/uar\/a2ui\/schemas(?:\?.*)?$/, async (route) => {
    await route.fulfill({ contentType: "application/json", body: "[]" });
  });
  await page.route(/\/api\/config\/persistence(?:\?.*)?$/, async (route) => {
    const request = route.request();
    if (request.method() !== "GET") {
      durableSettingsWrites.push({
        method: request.method(),
        path: new URL(request.url()).pathname,
      });
      await route.fulfill({
        status: 409,
        contentType: "application/json",
        body: JSON.stringify({ error: "Unexpected persistence write" }),
      });
      return;
    }
    await route.fulfill({ status: 204 });
  });

  return durableSettingsWrites;
}

async function contentBoxWidth(locator: Locator) {
  return locator.evaluate((element) => {
    const style = getComputedStyle(element);
    const horizontalPadding =
      Number.parseFloat(style.paddingLeft) + Number.parseFloat(style.paddingRight);
    return element.clientWidth - horizontalPadding;
  });
}

async function setContentBoxWidth(locator: Locator, width: number) {
  await locator.evaluate((element, targetWidth) => {
    const htmlElement = element as HTMLElement;
    const style = getComputedStyle(htmlElement);
    const horizontalPadding =
      Number.parseFloat(style.paddingLeft) + Number.parseFloat(style.paddingRight);
    const currentContentWidth = htmlElement.clientWidth - horizontalPadding;
    const currentBorderBoxWidth = htmlElement.getBoundingClientRect().width;
    htmlElement.style.flex = "0 0 auto";
    htmlElement.style.inlineSize = `${currentBorderBoxWidth + targetWidth - currentContentWidth}px`;
    htmlElement.style.maxInlineSize = "none";
  }, width);
  await expect.poll(() => contentBoxWidth(locator)).toBeCloseTo(width, 0);
}

async function computedTrackCount(grid: Locator) {
  return grid.evaluate((element) => {
    const columns = getComputedStyle(element).gridTemplateColumns.trim();
    return columns && columns !== "none" ? columns.split(/\s+/).length : 0;
  });
}

async function expectInside(inner: Locator, outer: Locator) {
  const [innerBox, outerBox] = await Promise.all([
    inner.boundingBox(),
    outer.boundingBox(),
  ]);
  expect(innerBox).not.toBeNull();
  expect(outerBox).not.toBeNull();
  if (!innerBox || !outerBox) return;

  expect(innerBox.x).toBeGreaterThanOrEqual(outerBox.x - overflowTolerance);
  expect(innerBox.x + innerBox.width).toBeLessThanOrEqual(
    outerBox.x + outerBox.width + overflowTolerance,
  );
  expect(innerBox.y).toBeGreaterThanOrEqual(outerBox.y - overflowTolerance);
  expect(innerBox.y + innerBox.height).toBeLessThanOrEqual(
    outerBox.y + outerBox.height + overflowTolerance,
  );
}

async function expectNoHorizontalOverflow(page: Page, card: Locator) {
  const pageOverflow = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(pageOverflow.scrollWidth).toBeLessThanOrEqual(
    pageOverflow.clientWidth + overflowTolerance,
  );

  const cardOverflow = await card.evaluate((element) => ({
    clientWidth: element.clientWidth,
    scrollWidth: element.scrollWidth,
  }));
  expect(cardOverflow.scrollWidth).toBeLessThanOrEqual(
    cardOverflow.clientWidth + overflowTolerance,
  );
}

async function readFocusStyle(control: Locator) {
  return control.evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      boxShadow: style.boxShadow,
      outlineColor: style.outlineColor,
      outlineStyle: style.outlineStyle,
      outlineWidth: Number.parseFloat(style.outlineWidth),
    };
  });
}

async function expectVisibleKeyboardFocus(
  control: Locator,
  scrollport: Locator,
  unfocusedStyle: Awaited<ReturnType<typeof readFocusStyle>>,
) {
  await expect(control).toBeFocused();
  await expectInside(control, scrollport);
  const focusStyle = await readFocusStyle(control);
  const hasOutline =
    focusStyle.outlineStyle !== "none" && focusStyle.outlineWidth > 0;
  const outlineChanged =
    hasOutline &&
    (focusStyle.outlineColor !== unfocusedStyle.outlineColor ||
      focusStyle.outlineStyle !== unfocusedStyle.outlineStyle ||
      focusStyle.outlineWidth !== unfocusedStyle.outlineWidth);
  const shadowChanged =
    focusStyle.boxShadow !== "none" &&
    focusStyle.boxShadow !== unfocusedStyle.boxShadow;
  expect(outlineChanged || shadowChanged).toBe(true);
}

async function expectKeyboardPath(
  page: Page,
  controls: Locator[],
  scrollport: Locator,
) {
  const firstUnfocusedStyle = await readFocusStyle(controls[0]);
  const originalBodyTabIndex = await page.evaluate(() => {
    const original = document.body.getAttribute("tabindex");
    document.body.tabIndex = -1;
    document.body.focus();
    return original;
  });
  let reachedFirstControl = false;
  try {
    for (let step = 0; step < 100; step += 1) {
      await page.keyboard.press("Tab");
      if (
        await controls[0].evaluate(
          (element) => document.activeElement === element,
        )
      ) {
        reachedFirstControl = true;
        break;
      }
    }
  } finally {
    await page.evaluate((original) => {
      if (original === null) document.body.removeAttribute("tabindex");
      else document.body.setAttribute("tabindex", original);
    }, originalBodyTabIndex);
  }
  expect(reachedFirstControl).toBe(true);
  await expectVisibleKeyboardFocus(
    controls[0],
    scrollport,
    firstUnfocusedStyle,
  );
  for (const control of controls.slice(1)) {
    const unfocusedStyle = await readFocusStyle(control);
    await page.keyboard.press("Tab");
    await expectVisibleKeyboardFocus(control, scrollport, unfocusedStyle);
  }
}

async function expectKeyboardOperability(
  page: Page,
  enable: Locator,
  baseUrl: Locator,
  protocol: Locator,
  apiKey: Locator,
  reveal: Locator,
  defaultModel: Locator,
  dirtyValue: string,
) {
  const expectOpenPopupInsideViewport = async () => {
    const popup = page.getByRole("listbox");
    await expect(popup).toBeVisible();
    const [popupBox, viewport] = await Promise.all([
      popup.boundingBox(),
      Promise.resolve(page.viewportSize()),
    ]);
    expect(popupBox).not.toBeNull();
    expect(viewport).not.toBeNull();
    if (!popupBox || !viewport) return;

    expect(popupBox.width).toBeGreaterThan(0);
    expect(popupBox.height).toBeGreaterThan(0);
    expect(popupBox.x).toBeGreaterThanOrEqual(0);
    expect(popupBox.y).toBeGreaterThanOrEqual(0);
    expect(popupBox.x + popupBox.width).toBeLessThanOrEqual(viewport.width);
    expect(popupBox.y + popupBox.height).toBeLessThanOrEqual(viewport.height);
  };

  await enable.focus();
  await expect(enable).toBeChecked();
  await page.keyboard.press("Space");
  await expect(enable).not.toBeChecked();
  await page.keyboard.press("Space");
  await expect(enable).toBeChecked();

  await baseUrl.focus();
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.type(dirtyValue);
  await expect(baseUrl).toHaveValue(dirtyValue);

  await protocol.focus();
  await page.keyboard.press("Space");
  await expect(protocol).toHaveAttribute("aria-expanded", "true");
  await expectOpenPopupInsideViewport();
  await page.keyboard.press("Escape");
  await expect(protocol).toHaveAttribute("aria-expanded", "false");

  await apiKey.focus();
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.type("responsive-fixture-key");
  await expect(apiKey).toHaveValue("responsive-fixture-key");

  await reveal.focus();
  await page.keyboard.press("Enter");
  await expect(apiKey).toHaveAttribute("type", "text");
  await page.keyboard.press("Enter");
  await expect(apiKey).toHaveAttribute("type", "password");

  await defaultModel.focus();
  await page.keyboard.press("Space");
  await expect(defaultModel).toHaveAttribute("aria-expanded", "true");
  await expectOpenPopupInsideViewport();
  await page.keyboard.press("Escape");
  await expect(defaultModel).toHaveAttribute("aria-expanded", "false");
}

async function expectFieldGeometry(
  baseUrl: Locator,
  protocol: Locator,
  layout: "stacked" | "same-row",
) {
  const [baseUrlBox, protocolBox] = await Promise.all([
    baseUrl.boundingBox(),
    protocol.boundingBox(),
  ]);
  expect(baseUrlBox).not.toBeNull();
  expect(protocolBox).not.toBeNull();
  if (!baseUrlBox || !protocolBox) return;

  if (layout === "stacked") {
    expect(protocolBox.y).toBeGreaterThanOrEqual(
      baseUrlBox.y + baseUrlBox.height - overflowTolerance,
    );
    return;
  }

  expect(protocolBox.y).toBeCloseTo(baseUrlBox.y, 0);
  expect(protocolBox.x).toBeGreaterThanOrEqual(
    baseUrlBox.x + baseUrlBox.width - overflowTolerance,
  );
}

test("provider fields respond to provider-panel width without losing keyboard or draft state", async ({
  page,
}) => {
  const durableSettingsWrites = await installDeterministicRoutes(page);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/admin/settings", { waitUntil: "domcontentloaded" });

  const providerCard = page.getByRole("group", { name: providerName });
  await expect(providerCard).toBeVisible();
  const providerList = providerCard.locator("xpath=..");
  await expect(providerList.getByRole("group")).toHaveCount(1);

  const enable = providerCard.getByRole("switch", {
    name: `Enable ${providerName} provider`,
  });
  const baseUrl = providerCard.getByLabel("Base URL");
  const protocol = providerCard.getByLabel("Protocol");
  const apiKey = providerCard.getByRole("textbox", {
    name: "API Key",
    exact: true,
  });
  const reveal = providerCard.getByRole("button", {
    name: `Show ${providerName} API key`,
  });
  const defaultModel = providerCard.getByRole("combobox", {
    name: `${providerName} default model`,
  });
  const grid = baseUrl.locator(
    "xpath=ancestor::div[contains(concat(' ', normalize-space(@class), ' '), ' grid ')][1]",
  );
  const controls = [enable, baseUrl, protocol, apiKey, reveal, defaultModel];

  const rootFontSize = await page.locator("html").evaluate((element) =>
    Number.parseFloat(getComputedStyle(element).fontSize),
  );
  const boundary = rootFontSize * 36;
  const originalStyle = await providerList.getAttribute("style");

  try {
    await setContentBoxWidth(providerList, boundary - 1);
    expect(await computedTrackCount(grid)).toBe(1);
    await expectNoHorizontalOverflow(page, providerCard);
    for (const control of controls) await expectInside(control, providerCard);
    await expectFieldGeometry(baseUrl, protocol, "stacked");

    const dirtyValue =
      "https://edited.responsive-fixture.example.test/v1/organizations/long-context-draft";
    await expectKeyboardOperability(
      page,
      enable,
      baseUrl,
      protocol,
      apiKey,
      reveal,
      defaultModel,
      dirtyValue,
    );
    await expect(providerCard.getByText("Modified", { exact: true })).toBeVisible();

    await expectKeyboardPath(page, controls, providerList);

    await setContentBoxWidth(providerList, boundary);
    await expect.poll(() => contentBoxWidth(providerList)).toBe(boundary);
    expect(await computedTrackCount(grid)).toBe(2);
    await expect(baseUrl).toHaveValue(dirtyValue);
    await expectFieldGeometry(baseUrl, protocol, "same-row");

    await setContentBoxWidth(providerList, boundary + 1);
    expect(await computedTrackCount(grid)).toBe(2);
    await expect(baseUrl).toHaveValue(dirtyValue);
    await expect(providerCard.getByText("Modified", { exact: true })).toBeVisible();
    await expectNoHorizontalOverflow(page, providerCard);
    for (const control of controls) await expectInside(control, providerCard);
    await expectFieldGeometry(baseUrl, protocol, "same-row");
    await expectKeyboardPath(page, controls, providerList);

    await providerList.evaluate((element, styleAttribute) => {
      if (styleAttribute === null) element.removeAttribute("style");
      else element.setAttribute("style", styleAttribute);
    }, originalStyle);
    await expect.poll(() => contentBoxWidth(providerList)).toBeGreaterThanOrEqual(boundary);
    expect(await computedTrackCount(grid)).toBe(2);
    await expect(baseUrl).toHaveValue(dirtyValue);
    await expect(providerCard.getByText("Modified", { exact: true })).toBeVisible();
    await expectNoHorizontalOverflow(page, providerCard);
    for (const control of controls) await expectInside(control, providerCard);
    await expectFieldGeometry(baseUrl, protocol, "same-row");
    await expectKeyboardPath(page, controls, providerList);

    await setContentBoxWidth(providerList, boundary - 1);
    expect(await computedTrackCount(grid)).toBe(1);
    await expect(baseUrl).toHaveValue(dirtyValue);
    await expect(providerCard.getByText("Modified", { exact: true })).toBeVisible();
    await expectFieldGeometry(baseUrl, protocol, "stacked");
  } finally {
    await providerList.evaluate((element, styleAttribute) => {
      if (styleAttribute === null) element.removeAttribute("style");
      else element.setAttribute("style", styleAttribute);
    }, originalStyle);
  }

  expect(durableSettingsWrites).toEqual([]);
});
