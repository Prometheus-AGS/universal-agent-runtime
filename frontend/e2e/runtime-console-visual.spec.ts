import { expect, type Locator, type Page, test } from "@playwright/test";

async function expectNoErrorBoundary(page: Page) {
  await expect(page.getByText("Oops! Page not found")).not.toBeVisible();
  await expect(page.getByText("Application error")).not.toBeVisible();
}

async function expectNoOverlap(first: Locator, second: Locator) {
  await expect(first).toBeVisible();
  await expect(second).toBeVisible();

  const [a, b] = await Promise.all([
    first.boundingBox(),
    second.boundingBox(),
  ]);

  expect(a, "first locator must have a bounding box").not.toBeNull();
  expect(b, "second locator must have a bounding box").not.toBeNull();

  if (!a || !b) return;

  const xOverlap = Math.max(
    0,
    Math.min(a.x + a.width, b.x + b.width) - Math.max(a.x, b.x),
  );
  const yOverlap = Math.max(
    0,
    Math.min(a.y + a.height, b.y + b.height) - Math.max(a.y, b.y),
  );

  expect(xOverlap * yOverlap).toBe(0);
}

test.describe("Runtime console visual verification", () => {
  test("desktop cockpit shell renders with stable context panels", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto("/admin/runtime");

    await expect(page.getByTestId("admin-shell")).toBeVisible();
    await expect(page.getByTestId("admin-navigation")).toBeVisible();
    await expect(page.getByTestId("admin-topbar")).toBeVisible();
    await expect(page.getByTestId("admin-section-runtime")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Live Runs" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Provider Health" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Workflow State" })).toBeVisible();
    await expect(page.getByText("No runtime runs observed yet")).toBeVisible();
    await expect(page.getByText("Provider health has not reported")).toBeVisible();

    await expectNoOverlap(
      page.getByTestId("admin-navigation"),
      page.getByTestId("admin-section-runtime"),
    );
    await expectNoOverlap(
      page.getByTestId("admin-topbar"),
      page.getByRole("heading", { name: "Live Runs" }),
    );
    await expectNoErrorBoundary(page);
  });

  test("desktop navigation reaches key runtime console surfaces", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto("/admin/runtime");

    const surfaces = [
      { id: "runs", path: /\/admin\/runs/, heading: "Runs" },
      { id: "approvals", path: /\/admin\/approvals/, heading: "Tool Approvals" },
      { id: "protocols", path: /\/admin\/protocols/, heading: "Compatibility Console" },
      { id: "providers", path: /\/admin\/providers/, heading: "Providers" },
      { id: "memory", path: /\/admin\/memory/, heading: "Memory Browser" },
      { id: "a2ui-testing", path: /\/admin\/a2ui-testing/, heading: "A2UI Schema Testing" },
    ] as const;

    for (const surface of surfaces) {
      await page.getByTestId(`admin-nav-${surface.id}`).click();
      await expect(page).toHaveURL(surface.path);
      await expect(page.getByTestId(`admin-section-${surface.id}`)).toBeVisible();
      await expect(
        page.getByRole("heading", { name: surface.heading }).first(),
      ).toBeVisible();
      await expectNoErrorBoundary(page);
    }
  });

  test("mobile navigation routes and clears the overlay", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/admin/runtime");

    await expect(page.getByTestId("admin-mobile-nav-trigger")).toBeVisible();
    await expect(page.getByTestId("admin-section-runtime")).toBeVisible();

    await page.getByTestId("admin-mobile-nav-trigger").click();
    await expect(page.getByTestId("admin-mobile-nav-overlay")).toBeVisible();
    await expect(page.getByTestId("admin-navigation")).toBeVisible();

    await page.getByTestId("admin-nav-protocols").click();
    await expect(page).toHaveURL(/\/admin\/protocols/);
    await expect(page.getByTestId("admin-section-protocols")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Compatibility Console" })).toBeVisible();
    await expect(page.getByTestId("admin-mobile-nav-overlay")).not.toBeVisible();

    await expectNoOverlap(
      page.getByTestId("admin-mobile-nav-trigger"),
      page.getByRole("heading", { name: "Compatibility Console" }),
    );
    await expectNoErrorBoundary(page);
  });

  test("command palette opens and routes to provider diagnostics", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto("/admin/runtime");

    await expect(page.getByTestId("admin-shell")).toBeVisible();
    await expect(page.getByTestId("admin-command-trigger")).toBeVisible();

    const commandInput = page.getByPlaceholder("Search runtime, providers, protocols...");
    await page.evaluate(() => {
      document.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: "k",
          ctrlKey: true,
          bubbles: true,
        }),
      );
    });
    await commandInput.waitFor({ state: "visible", timeout: 1000 }).catch(async () => {
      await page.getByTestId("admin-command-trigger").click();
    });
    await expect(commandInput).toBeVisible();

    await page
      .getByRole("dialog")
      .getByText("Providers", { exact: true })
      .click();
    await expect(page).toHaveURL(/\/admin\/providers/);
    await expect(page.getByTestId("admin-section-providers")).toBeVisible();
    await expect(page.getByText(/providers connect this runtime|select a provider|no providers configured/i).first()).toBeVisible();
    await expectNoErrorBoundary(page);
  });
});
