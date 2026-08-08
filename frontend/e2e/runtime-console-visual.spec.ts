import { expect, test } from "@chromatic-com/playwright";
import type { Locator, Page } from "@playwright/test";

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

    await expect(page.getByRole("navigation", { name: "Primary navigation" })).toBeVisible();
    await expect(page.getByRole("banner")).toBeVisible();
    await expect(page.getByTestId("admin-section-runtime")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Live Runs" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Provider Health" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Workflow State" })).toBeVisible();
    await expect(page.getByText("No runtime runs observed yet")).toBeVisible();
    await expect(page.getByText("No provider health reported yet")).toBeVisible();

    await expectNoOverlap(
      page.getByRole("navigation", { name: "Primary navigation" }),
      page.getByTestId("admin-section-runtime"),
    );
    await expectNoOverlap(
      page.getByRole("banner"),
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
      { id: "a2ui-testing", path: /\/admin\/a2ui-testing/, heading: "A2UI Live Testing" },
    ] as const;

    for (const surface of surfaces) {
      await page.goto(`/admin/${surface.id}`);
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

    await expect(page.getByRole("navigation", { name: "Compact navigation" })).toBeVisible();
    await expect(page.getByTestId("admin-section-runtime")).toBeVisible();

    await page.getByRole("button", { name: "Configure" }).click();
    const configureDialog = page.getByRole("dialog");
    await expect(configureDialog).toBeVisible();
    await configureDialog.getByRole("link", { name: /Runtime settings/ }).click();
    await expect(page).toHaveURL(/\/admin\/settings/);
    await expect(page.getByTestId("admin-section-settings")).toBeVisible();

    await expectNoOverlap(
      page.getByRole("button", { name: "Open command palette" }),
      page.getByTestId("admin-section-settings"),
    );
    await expectNoErrorBoundary(page);
  });

  test("command palette filters and routes with keyboard activation", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto("/admin/runtime");

    await expect(page.getByRole("navigation", { name: "Primary navigation" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Open command palette" })).toBeVisible();

    const commandInput = page.getByPlaceholder("Search routes and commands…");
    await page.getByRole("button", { name: "Open command palette" }).click();
    await expect(commandInput).toBeVisible();

    await commandInput.fill("credentials");
    await expect(
      page.getByRole("dialog").getByText("Providers", { exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("dialog").getByText("Skills", { exact: true }),
    ).not.toBeVisible();
    await commandInput.press("Enter");
    await expect(page).toHaveURL(/\/admin\/providers/);
    await expect(page.getByTestId("admin-section-providers")).toBeVisible();
    await expect(page.getByText(/providers connect this runtime|select a provider|no providers configured/i).first()).toBeVisible();
    await expectNoErrorBoundary(page);
  });
});
