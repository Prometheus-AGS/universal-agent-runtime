import { expect, test } from "@chromatic-com/playwright";

test.describe("Settings decomposition smoke", () => {
  test("desktop preserves settings navigation and custom panel composition", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto("/admin/settings");

    await expect(page.getByTestId("admin-section-settings")).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "LLM Providers" }),
    ).toBeVisible();

    await page.getByRole("button", { name: "Prompt Caching" }).click();
    await expect(
      page.getByRole("heading", { name: "Prompt Caching" }),
    ).toBeVisible();
    await expect(page.getByText("Application error")).not.toBeVisible();
  });

  test("mobile preserves the stacked settings surface", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/admin/settings");

    await expect(page.getByTestId("admin-section-settings")).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "LLM Providers" }),
    ).toBeVisible();
    await expect(page.getByRole("button", { name: "Memory" })).toBeVisible();
    await expect(page.getByText("Application error")).not.toBeVisible();
  });
});
