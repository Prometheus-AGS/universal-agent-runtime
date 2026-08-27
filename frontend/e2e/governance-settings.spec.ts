import { expect, test } from "@playwright/test";
import type { Locator, Page } from "@playwright/test";

type EffectiveState = "unknown" | "required" | "on" | "off";

interface GovernanceFixture {
  enabled: boolean;
  revision: number;
  state: EffectiveState;
  reasons: string[];
  statusAvailable: boolean;
  bootInstanceId?: string;
  confirmationGate?: {
    pending: boolean;
    requested: () => void;
    release: Promise<void>;
  };
}

const overflowTolerance = 1;

function setting(key: string, name: string, data: unknown) {
  return {
    id: `setting-${key}`,
    settings_type_id: "settings-type-governance",
    key,
    name,
    data,
    created_at: "2026-08-27T00:00:00Z",
    meta: { source: "Api", is_drift: false },
  };
}

function governanceSettings(fixture: GovernanceFixture) {
  return [
    setting("governance.enabled", "Enforce tool governance", fixture.enabled),
    setting("governance.default_mode", "Default authorization mode", "permit_all"),
    setting("governance.allowed_actions", "Globally allowed actions", []),
    setting("governance.policy_reload_enabled", "Hot policy reload", true),
  ];
}

function governanceStatus(fixture: GovernanceFixture) {
  const required = fixture.state === "required";
  const enabled = required || fixture.state !== "off";
  return {
    boot_instance_id: fixture.bootInstanceId ?? "governance-e2e-boot",
    revision: fixture.revision,
    phase: fixture.state === "unknown" ? "initializing" : enabled ? "on" : "off",
    effective_state: fixture.state,
    effective_enabled: enabled,
    may_disable: !required && fixture.state !== "unknown",
    mutation_available: fixture.state !== "unknown",
    configured_host: "127.0.0.1",
    bound_addresses: ["127.0.0.1:8080"],
    jwt_required: required,
    reasons: fixture.reasons,
  };
}

async function installDeterministicRoutes(
  page: Page,
  fixture: GovernanceFixture,
) {
  await page.route(/\/api\/uar\/settings(?:\/[^?]*)?(?:\?.*)?$/, async (route) => {
    const request = route.request();
    const pathname = new URL(request.url()).pathname;

    if (pathname === "/api/uar/settings/governance/status") {
      if (!fixture.statusAvailable) {
        await route.fulfill({ status: 503, body: "unavailable" });
        return;
      }
      if (fixture.confirmationGate?.pending) {
        fixture.confirmationGate.requested();
        await fixture.confirmationGate.release;
        fixture.confirmationGate.pending = false;
      }
      await route.fulfill({
        contentType: "application/json",
        body: JSON.stringify(governanceStatus(fixture)),
      });
      return;
    }

    if (pathname === "/api/uar/settings/governance" && request.method() === "PUT") {
      const body = request.postDataJSON() as { data: Record<string, unknown> };
      const submitted = body.data;
      if (typeof submitted.enabled === "boolean") {
        fixture.enabled = submitted.enabled;
        fixture.state = submitted.enabled ? "on" : "off";
        fixture.reasons = [];
      }
      fixture.revision += 1;
      if (fixture.confirmationGate) {
        fixture.bootInstanceId = "governance-e2e-confirmed-boot";
        fixture.confirmationGate.pending = true;
      }
      const status = governanceStatus(fixture);
      await route.fulfill({
        contentType: "application/json",
        body: JSON.stringify({
          status: "updated",
          results: Object.keys(submitted).map((key) => ({
            key: `governance.${key}`,
            status: "updated",
          })),
          applied_status: {
            boot_instance_id: status.boot_instance_id,
            revision: status.revision,
          },
          governance_status: status,
        }),
      });
      return;
    }

    if (request.method() !== "GET") {
      await route.fulfill({ status: 409, body: "unexpected settings write" });
      return;
    }

    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify(
        pathname === "/api/uar/settings/governance"
          ? governanceSettings(fixture)
          : [],
      ),
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
      body: JSON.stringify({ status: "ok", version: "governance-e2e" }),
    });
  });
  await page.route(/\/api\/uar\/resolve-model(?:\?.*)?$/, async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ ok: true, provider_id: "local", model_id: "local/test" }),
    });
  });
  await page.route(/\/api\/uar\/providers\/health(?:\?.*)?$/, async (route) => {
    await route.fulfill({ contentType: "application/json", body: "{\"providers\":{}}" });
  });
  await page.route(/\/api\/uar\/a2ui\/schemas(?:\?.*)?$/, async (route) => {
    await route.fulfill({ contentType: "application/json", body: "[]" });
  });
  await page.route(/\/api\/config\/persistence(?:\?.*)?$/, async (route) => {
    await route.fulfill({ status: 204 });
  });
}

async function openGovernance(
  page: Page,
  fixture: GovernanceFixture,
  options: { theme?: "light" | "dark"; width?: number } = {},
) {
  await installDeterministicRoutes(page, fixture);
  await page.setViewportSize({ width: options.width ?? 1024, height: 900 });
  await page.addInitScript(
    ({ theme }) => {
      localStorage.setItem("uar-theme", theme);
    },
    { theme: options.theme ?? "light" },
  );
  await page.emulateMedia({ colorScheme: options.theme === "dark" ? "dark" : "light" });
  await page.goto("/admin/settings", { waitUntil: "domcontentloaded" });
  await page.getByRole("button", { name: "Governance" }).click();
  await expect(page.getByRole("heading", { name: "Governance" })).toBeVisible();
}

async function expectNoHorizontalOverflow(
  page: Page,
  surfaces: Array<{ name: string; locator: Locator }>,
) {
  const documentOverflow = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(documentOverflow.scrollWidth).toBeLessThanOrEqual(
    documentOverflow.clientWidth + overflowTolerance,
  );
  for (const { name, locator } of surfaces) {
    const overflow = await locator.evaluate((element) => ({
      clientWidth: element.clientWidth,
      scrollWidth: element.scrollWidth,
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight,
      overflowY: getComputedStyle(element).overflowY,
    }));
    expect(overflow.scrollWidth, `${name} horizontal overflow`).toBeLessThanOrEqual(
      overflow.clientWidth + overflowTolerance,
    );
    if (overflow.overflowY === "hidden" || overflow.overflowY === "clip") {
      expect(overflow.scrollHeight, `${name} vertical clipping`).toBeLessThanOrEqual(
        overflow.clientHeight + overflowTolerance,
      );
    }
  }
}

async function tabUntilFocused(page: Page, target: Locator) {
  for (let step = 0; step < 80; step += 1) {
    await page.keyboard.press("Tab");
    if (await target.evaluate((element) => element === document.activeElement)) {
      return;
    }
  }
  throw new Error("Keyboard traversal did not reach the governance switch");
}

async function expectVisibleWithoutClipping(target: Locator) {
  const result = await target.evaluate((element) => {
    const targetRect = element.getBoundingClientRect();
    const viewport = {
      top: 0,
      left: 0,
      right: window.innerWidth,
      bottom: window.innerHeight,
    };
    const clippedAncestors: string[] = [];
    let ancestor = element.parentElement;
    while (ancestor) {
      const style = getComputedStyle(ancestor);
      if (
        style.overflowX === "hidden" ||
        style.overflowX === "clip" ||
        style.overflowY === "hidden" ||
        style.overflowY === "clip"
      ) {
        const ancestorRect = ancestor.getBoundingClientRect();
        if (
          targetRect.left < ancestorRect.left ||
          targetRect.right > ancestorRect.right ||
          targetRect.top < ancestorRect.top ||
          targetRect.bottom > ancestorRect.bottom
        ) {
          clippedAncestors.push(
            `${ancestor.tagName.toLowerCase()}.${ancestor.className}`,
          );
        }
      }
      ancestor = ancestor.parentElement;
    }
    return {
      withinViewport:
        targetRect.left >= viewport.left &&
        targetRect.right <= viewport.right &&
        targetRect.top >= viewport.top &&
        targetRect.bottom <= viewport.bottom,
      clippedAncestors,
    };
  });
  expect(result.withinViewport).toBe(true);
  expect(result.clippedAncestors).toEqual([]);
}

async function contrastRatio(foreground: Locator, background: Locator) {
  return foreground.evaluate((element, backgroundElement) => {
    const parse = (value: string) => {
      const canvas = document.createElement("canvas");
      canvas.width = 1;
      canvas.height = 1;
      const context = canvas.getContext("2d", { willReadFrequently: true });
      if (!context) throw new Error("Canvas color parser is unavailable");
      context.clearRect(0, 0, 1, 1);
      context.fillStyle = value;
      context.fillRect(0, 0, 1, 1);
      const channels = context.getImageData(0, 0, 1, 1).data;
      return {
        red: channels[0] / 255,
        green: channels[1] / 255,
        blue: channels[2] / 255,
        alpha: channels[3] / 255,
      };
    };
    const compositeOver = (
      top: ReturnType<typeof parse>,
      bottom: ReturnType<typeof parse>,
    ) => {
      const alpha = top.alpha + bottom.alpha * (1 - top.alpha);
      const channel = (topValue: number, bottomValue: number) =>
        alpha === 0
          ? 0
          : (topValue * top.alpha + bottomValue * bottom.alpha * (1 - top.alpha)) /
            alpha;
      return {
        red: channel(top.red, bottom.red),
        green: channel(top.green, bottom.green),
        blue: channel(top.blue, bottom.blue),
        alpha,
      };
    };
    const effectiveBackground = (start: Element) => {
      let result = { red: 0, green: 0, blue: 0, alpha: 0 };
      let cursor: Element | null = start;
      while (cursor && result.alpha < 0.999) {
        const layer = parse(getComputedStyle(cursor).backgroundColor);
        result = compositeOver(result, layer);
        cursor = cursor.parentElement;
      }
      return result.alpha < 0.999
        ? compositeOver(result, { red: 1, green: 1, blue: 1, alpha: 1 })
        : result;
    };
    const luminance = (color: ReturnType<typeof parse>) => {
      const linear = [color.red, color.green, color.blue].map((channel) =>
        channel <= 0.04045
          ? channel / 12.92
          : ((channel + 0.055) / 1.055) ** 2.4,
      );
      return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
    };
    const backgroundColor = effectiveBackground(backgroundElement as Element);
    const foregroundColor = compositeOver(
      parse(getComputedStyle(element).color),
      backgroundColor,
    );
    const foregroundLuminance = luminance(foregroundColor);
    const backgroundLuminance = luminance(backgroundColor);
    const lighter = Math.max(foregroundLuminance, backgroundLuminance);
    const darker = Math.min(foregroundLuminance, backgroundLuminance);
    return (lighter + 0.05) / (darker + 0.05);
  }, await background.elementHandle());
}

async function borderContrastRatio(surface: Locator) {
  return surface.evaluate((element) => {
    const parse = (value: string) => {
      const canvas = document.createElement("canvas");
      canvas.width = 1;
      canvas.height = 1;
      const context = canvas.getContext("2d", { willReadFrequently: true });
      if (!context) throw new Error("Canvas color parser is unavailable");
      context.clearRect(0, 0, 1, 1);
      context.fillStyle = value;
      context.fillRect(0, 0, 1, 1);
      const channels = context.getImageData(0, 0, 1, 1).data;
      return {
        red: channels[0] / 255,
        green: channels[1] / 255,
        blue: channels[2] / 255,
        alpha: channels[3] / 255,
      };
    };
    const compositeOver = (
      top: ReturnType<typeof parse>,
      bottom: ReturnType<typeof parse>,
    ) => {
      const alpha = top.alpha + bottom.alpha * (1 - top.alpha);
      const channel = (topValue: number, bottomValue: number) =>
        alpha === 0
          ? 0
          : (topValue * top.alpha + bottomValue * bottom.alpha * (1 - top.alpha)) /
            alpha;
      return {
        red: channel(top.red, bottom.red),
        green: channel(top.green, bottom.green),
        blue: channel(top.blue, bottom.blue),
        alpha,
      };
    };
    const effectiveBackground = (start: Element | null) => {
      let result = { red: 0, green: 0, blue: 0, alpha: 0 };
      let cursor = start;
      while (cursor && result.alpha < 0.999) {
        result = compositeOver(
          result,
          parse(getComputedStyle(cursor).backgroundColor),
        );
        cursor = cursor.parentElement;
      }
      return result.alpha < 0.999
        ? compositeOver(result, { red: 1, green: 1, blue: 1, alpha: 1 })
        : result;
    };
    const luminance = (color: ReturnType<typeof parse>) => {
      const linear = [color.red, color.green, color.blue].map((channel) =>
        channel <= 0.04045
          ? channel / 12.92
          : ((channel + 0.055) / 1.055) ** 2.4,
      );
      return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
    };
    const style = getComputedStyle(element);
    const adjacentColor = effectiveBackground(element.parentElement);
    const border = luminance(compositeOver(parse(style.borderTopColor), adjacentColor));
    const adjacent = luminance(adjacentColor);
    return (Math.max(border, adjacent) + 0.05) / (Math.min(border, adjacent) + 0.05);
  });
}

for (const theme of ["light", "dark"] as const) {
  test(`${theme} Off warning remains readable and contained at 320 CSS px`, async ({ page }) => {
    const fixture: GovernanceFixture = {
      enabled: false,
      revision: 1,
      state: "off",
      reasons: [],
      statusAvailable: true,
    };
    await openGovernance(page, fixture, { theme, width: 320 });

    const warningText = page.getByText(
      "All available tools can run without Cedar policies, run-policy restrictions, or approval prompts.",
      { exact: true },
    );
    const warning = warningText.locator("..");
    const masterCard = page.getByRole("region", {
      name: "Enforce tool governance",
    });
    const masterGrid = masterCard.locator(":scope > div").first();
    const master = page.getByRole("switch", { name: "Enforce tool governance" });
    const fieldset = page.getByRole("group", { name: "Policy behavior when governance is on" });

    await expect(warningText).toBeVisible();
    await expect(master).toBeVisible();
    expect(
      await masterGrid.evaluate((element) =>
        getComputedStyle(element).gridTemplateColumns.trim().split(/\s+/),
      ),
    ).toHaveLength(1);
    const [masterBox, cardBox] = await Promise.all([
      master.boundingBox(),
      masterCard.boundingBox(),
    ]);
    expect(masterBox).not.toBeNull();
    expect(cardBox).not.toBeNull();
    expect(masterBox!.x).toBeGreaterThanOrEqual(cardBox!.x);
    expect(masterBox!.x + masterBox!.width).toBeLessThanOrEqual(
      cardBox!.x + cardBox!.width,
    );
    expect(await contrastRatio(warningText, warning)).toBeGreaterThanOrEqual(4.5);
    expect(await borderContrastRatio(warning)).toBeGreaterThanOrEqual(3);
    await expectNoHorizontalOverflow(page, [
      { name: "master card", locator: masterCard },
      { name: "warning", locator: warning },
      { name: "fieldset", locator: fieldset },
    ]);
  });
}

test("320 effective CSS px (640 at 200% zoom) preserves wrapped focus visibility and keyboard operation", async ({ page }) => {
  const fixture: GovernanceFixture = {
    enabled: false,
    revision: 1,
    state: "off",
    reasons: [],
    statusAvailable: true,
  };
  await openGovernance(page, fixture, { theme: "dark", width: 320 });

  const master = page.getByRole("switch", { name: "Enforce tool governance" });
  const masterCard = page.getByRole("region", {
    name: "Enforce tool governance",
  });
  expect(await page.evaluate(() => window.innerWidth)).toBe(320);
  const unfocusedShadow = await master.evaluate(
    (element) => getComputedStyle(element).boxShadow,
  );
  await page.evaluate(() => {
    document.body.tabIndex = -1;
    document.body.focus();
  });
  await tabUntilFocused(page, master);
  await expect(master).toBeFocused();
  const focusStyle = await master.evaluate((element) => getComputedStyle(element).boxShadow);
  expect(focusStyle).not.toBe("none");
  expect(focusStyle).not.toBe(unfocusedShadow);
  await expectVisibleWithoutClipping(master);
  await page.keyboard.press("Space");
  await expect(master).toBeChecked();

  const fieldset = page.getByRole("group", { name: "Policy behavior when governance is on" });
  const warning = page.getByRole("note");
  await expect(warning).toBeVisible();
  await expectNoHorizontalOverflow(page, [
    { name: "master card", locator: masterCard },
    { name: "warning", locator: warning },
    { name: "fieldset", locator: fieldset },
  ]);
});

test("Required and Unknown states expose locked reasons and recover through Refresh", async ({ page }) => {
  const fixture: GovernanceFixture = {
    enabled: true,
    revision: 3,
    state: "required",
    reasons: ["jwt_required", "configured_host_not_allowed"],
    statusAvailable: true,
  };
  await openGovernance(page, fixture);

  const master = page.getByRole("switch", { name: "Enforce tool governance" });
  await expect(master).toBeChecked();
  await expect(master).toHaveAttribute("aria-disabled", "true");
  await expect(master).toHaveAccessibleDescription(/JWT authentication is active/);
  await expect(master).toHaveAccessibleDescription(
    /configured listener host is not localhost or 127\.0\.0\.1/,
  );
  await master.focus();
  await page.keyboard.press("Space");
  await page.keyboard.press("Enter");
  await expect(master).toBeChecked();

  fixture.statusAvailable = false;
  await page.getByRole("button", { name: "Refresh" }).click();
  await expect(page.getByText("Unknown", { exact: true })).toBeVisible();
  await expect(page.getByText("Unavailable", { exact: true })).toBeVisible();
  await expect(page.getByRole("switch", { name: "Enforce tool governance" })).toHaveCount(0);

  fixture.statusAvailable = true;
  fixture.state = "on";
  fixture.reasons = [];
  fixture.revision += 1;
  await page.getByRole("button", { name: "Refresh" }).click();
  await expect(page.getByText("On", { exact: true })).toBeVisible();
});

test("save reports the authoritative Off transition before success", async ({ page }) => {
  let releaseConfirmation!: () => void;
  let markConfirmationRequested!: () => void;
  const confirmationRequested = new Promise<void>((resolve) => {
    markConfirmationRequested = resolve;
  });
  const confirmationRelease = new Promise<void>((resolve) => {
    releaseConfirmation = resolve;
  });
  const fixture: GovernanceFixture = {
    enabled: true,
    revision: 8,
    state: "on",
    reasons: [],
    statusAvailable: true,
    confirmationGate: {
      pending: false,
      requested: markConfirmationRequested,
      release: confirmationRelease,
    },
  };
  await openGovernance(page, fixture);

  const master = page.getByRole("switch", { name: "Enforce tool governance" });
  await master.click();
  await expect(page.getByText(/After Save, all available tools can run/)).toBeVisible();
  await expect(page.getByText("On", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Save" }).click();

  await confirmationRequested;
  await expect(page.getByText("Off", { exact: true })).toBeVisible();
  await expect(
    page
      .locator("p:not(.sr-only)")
      .filter({ hasText: "Governance settings saved and confirmed." }),
  ).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Save" })).toBeDisabled();
  releaseConfirmation();
  await expect(page.getByText("Off", { exact: true })).toBeVisible();
  await expect(
    page
      .locator("p:not(.sr-only)")
      .filter({ hasText: "Governance settings saved and confirmed." }),
  ).toBeVisible();
  await expect(page.getByRole("switch", { name: "Enforce tool governance" })).not.toBeChecked();
});
