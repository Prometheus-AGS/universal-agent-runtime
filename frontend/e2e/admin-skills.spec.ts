import { test, expect } from "@playwright/test";

test.describe("Admin — Skills page", () => {
  test.beforeEach(async ({ page }) => {
    await page.route("**/api/skills", async (route) => {
      await route.fulfill({ json: { skills: [] } });
    });
    await page.goto("/admin/skills");
  });

  test("skills page loads", async ({ page }) => {
    await expect(page.locator("body")).toBeVisible();
    await expect(page).toHaveURL(/\/admin/);
  });

  test("admin sidebar contains Skills link", async ({ page }) => {
    await page.goto("/admin");
    await expect(page.locator("text=Skills").first()).toBeVisible();
  });

  test("skills list or empty state is visible", async ({ page }) => {
    await expect(page.getByText("no skills configured", { exact: true })).toBeVisible({
      timeout: 15000,
    });
  });

  test("new skill dialog includes model override selector", async ({ page }) => {
    const newBtn = page.locator("button:has-text('New Skill')").first();
    await expect(newBtn).toBeVisible({ timeout: 15000 });
    await newBtn.click();

    // Dialog should appear
    const dialog = page.locator("[role='dialog']").first();
    await expect(dialog).toBeVisible({ timeout: 15000 });

    // Model override label should be visible
    const modelLabel = dialog.locator("text=Model override");
    await expect(modelLabel).toBeVisible({ timeout: 15000 });
  });

  test("new skill dialog has required title field", async ({ page }) => {
    const newBtn = page.locator("button:has-text('New Skill')").first();
    await newBtn.click();

    const dialog = page.locator("[role='dialog']").first();
    await expect(dialog).toBeVisible({ timeout: 15000 });

    const titleInput = dialog.locator("input#skill-title, input[placeholder='Customer Success Coach']").first();
    await expect(titleInput).toBeVisible({ timeout: 3000 });
  });
});

/**
 * R3: the UI must SHOW builtin skills and administer them correctly.
 *
 * The suite above mocks an EMPTY skills list, so no row is ever rendered and
 * nothing about builtin handling is exercised. These tests mock a response
 * containing one builtin and one user skill, which is the only way to assert
 * the rules that matter:
 *
 *   - a builtin is visually distinguishable from a user skill
 *   - its delete affordance is DISABLED, not a button that 409s
 *   - its toggle still works, because disabling is allowed and deleting is not
 *
 * That last pair is the substance of R2 as the user experiences it. A UI that
 * offers Delete and then fails is worse than one that does not offer it.
 */
test.describe("Admin — Skills page, builtin handling (R3)", () => {
  const BUILTIN = {
    skill_id: "pack-builtin-skill",
    title: "Pack Builtin Skill",
    description: "Ships with the skill pack",
    origin: "builtin",
    enabled: true,
    version: "1.0.0",
  };
  const USER = {
    skill_id: "user-made-skill",
    title: "User Made Skill",
    description: "Created by the operator",
    origin: "user",
    enabled: true,
    version: "1.0.0",
  };

  test.beforeEach(async ({ page }) => {
    await page.route("**/api/skills", async (route) => {
      await route.fulfill({ json: { skills: [BUILTIN, USER] } });
    });
    await page.goto("/admin/skills");
  });

  test("a builtin skill is visually marked and a user skill is not", async ({ page }) => {
    await expect(page.getByText(BUILTIN.title).first()).toBeVisible({ timeout: 15000 });
    await expect(page.getByText(USER.title).first()).toBeVisible();

    // The badge is the distinguishing mark. Without it an operator cannot tell
    // which skills they are allowed to remove.
    await expect(page.getByText("built-in").first()).toBeVisible();
  });

  test("delete is DISABLED for a builtin, enabled for a user skill", async ({ page }) => {
    const builtinDelete = page.getByRole("button", { name: `Delete ${BUILTIN.title}` });
    const userDelete = page.getByRole("button", { name: `Delete ${USER.title}` });

    await expect(builtinDelete).toBeVisible({ timeout: 15000 });
    await expect(builtinDelete).toBeDisabled();

    // The contrast is the assertion: a blanket-disabled Delete column would
    // pass a one-sided check while breaking normal skill management.
    await expect(userDelete).toBeEnabled();
  });

  test("a builtin skill can still be toggled — disable is allowed, delete is not", async ({
    page,
  }) => {
    const toggle = page.getByRole("button", { name: `Disable ${BUILTIN.title}` });
    await expect(toggle).toBeVisible({ timeout: 15000 });
    await expect(toggle).toBeEnabled();
  });
});
