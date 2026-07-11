import { expect, type Page, test } from "@playwright/test";

type RuntimeReplayWindow = Window & {
  __uarRuntimeReplay?: {
    reset: () => void;
    replayAll: () => void;
    replayUpdates: () => void;
    replayApprovalStatus: (status: "approved" | "denied" | "expired") => void;
    snapshot: () => Record<string, number>;
  };
};

async function replayAll(page: Page) {
  await page.waitForFunction(() => Boolean((window as RuntimeReplayWindow).__uarRuntimeReplay));
  return page.evaluate(() => {
    const helper = (window as RuntimeReplayWindow).__uarRuntimeReplay;
    if (!helper) throw new Error("runtime replay helper not installed");
    helper.reset();
    helper.replayAll();
    return helper.snapshot();
  });
}

test.describe("Runtime event replay entity sync", () => {
  test("cockpit updates replayed runtime state without refresh", async ({ page }) => {
    await page.goto("/admin/runtime");
    await replayAll(page);

    await expect(page.getByText("Live Replay Run")).toBeVisible();
    await expect(page.getByText("Replay tool execution")).toBeVisible();
    await expect(page.getByText("openai")).toBeVisible();
    await expect(page.getByText("42ms")).toBeVisible();
    await expect(page.getByText("workflow_mirror")).toBeVisible();
    await expect(page.getByText("Mirrored KBD waypoint for surreal-memory-workflow-mirror-tests")).toBeVisible();
    await expect(page.getByText("No runtime runs observed yet")).not.toBeVisible();
    await expect(page.getByText("No provider health reported yet")).not.toBeVisible();
  });

  test("runs detail updates replayed artifacts and tool calls without refresh", async ({ page }) => {
    await page.goto("/admin/runs");
    await replayAll(page);

    await expect(page.getByRole("heading", { name: "Live Replay Run" })).toBeVisible();
    await expect(page.getByText("Replay Diagnostics Artifact")).toBeVisible();
    await expect(page.getByText("provider.health.check")).toBeVisible();
    await expect(page.getByText("json · application/json")).toBeVisible();
  });

  test("approvals surface updates replayed approval states without refresh", async ({ page }) => {
    await page.goto("/admin/approvals");
    await replayAll(page);

    await expect(page.getByText("replay-approval-001")).toBeVisible();
    await expect(page.getByText("run replay-run-001 · tool replay-tool-001")).toBeVisible();
    await expect(page.getByText("1 pending")).toBeVisible();

    await page.evaluate(() => {
      const helper = (window as RuntimeReplayWindow).__uarRuntimeReplay;
      if (!helper) throw new Error("runtime replay helper not installed");
      helper.replayApprovalStatus("approved");
    });

    await expect(page.getByText("0 pending")).toBeVisible();
    await expect(page.getByText("approved")).toBeVisible();
  });

  test("protocol console updates replayed AG-UI, A2UI, and routing state without refresh", async ({ page }) => {
    await page.goto("/admin/protocols");
    const snapshot = await replayAll(page);

    expect(snapshot.RuntimeAgUiEvent).toBe(2);
    await expect(page.getByText("2 events")).toBeVisible();
    await expect(page.getByText("1 surfaces")).toBeVisible();
    await expect(page.getByText("1 route decisions")).toBeVisible();
    await expect(page.getByText("tool_call.delta")).toBeVisible();
    await expect(page.getByText("Replay A2UI Surface")).toBeVisible();
    await expect(page.getByText("Replay payload rendered from entity graph")).toBeVisible();
    await expect(page.getByText("gpt-5.4")).toBeVisible();
    await expect(page.getByText("Replay route selected for tool-capable runtime validation")).toBeVisible();

    await page.evaluate(() => {
      const helper = (window as RuntimeReplayWindow).__uarRuntimeReplay;
      if (!helper) throw new Error("runtime replay helper not installed");
      helper.replayUpdates();
    });

    await expect(page.getByText("Replay payload updated from chunk")).toBeVisible();
    await expect(page.getByText("Replay payload rendered from entity graph")).not.toBeVisible();
  });

  test("existing empty runtime state remains valid without replay", async ({ page }) => {
    await page.goto("/admin/runtime");

    await expect(page.getByText("No runtime runs observed yet")).toBeVisible();
    await expect(page.getByText("No provider health reported yet")).toBeVisible();
  });
});
