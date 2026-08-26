import { expect, test } from "@playwright/test";

interface ObservedRequest {
  method: string;
  path: string;
  status?: number;
}

test("installed session prompt-caching override persists without app errors", async ({
  page,
  request,
}) => {
  const observed: ObservedRequest[] = [];
  const consoleErrors: string[] = [];
  const pageErrors: string[] = [];

  page.on("request", (entry) => {
    const url = new URL(entry.url());
    if (
      url.origin === "http://127.0.0.1:1906" &&
      /\/api\/uar\/sessions\/[^/]+\/(agent-config|prompt-caching)$/.test(
        url.pathname,
      )
    ) {
      observed.push({ method: entry.method(), path: url.pathname });
    }
  });
  page.on("response", (entry) => {
    const url = new URL(entry.url());
    const record = [...observed]
      .reverse()
      .find(
        (candidate) =>
          candidate.method === entry.request().method() &&
          candidate.path === url.pathname,
      );
    if (record) record.status = entry.status();
  });
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  page.on("pageerror", (error) => pageErrors.push(error.message));

  await page.goto("/threads", { waitUntil: "domcontentloaded" });
  await page.getByRole("button", { name: "New thread" }).click();
  const configButton = page.getByRole("button", {
    name: "Session configuration",
  });
  await expect(configButton).toBeEnabled();
  await configButton.click();

  await expect(
    page.getByRole("heading", { name: "Session Configuration" }),
  ).toBeVisible();
  const promptCaching = page.getByLabel("Prompt Caching");
  await expect(promptCaching).toBeEnabled();
  await expect(promptCaching).toContainText("Inherit");
  await expect(
    page.getByRole("status").filter({ hasText: "Effective now" }),
  ).toContainText("Effective now: Off from global default");

  await promptCaching.click();
  await page.getByRole("option", { name: "On", exact: true }).click();
  const saveResponsePromise = page.waitForResponse(
    (response) =>
      response.request().method() === "POST" &&
      /\/api\/uar\/sessions\/[^/]+\/agent-config$/.test(
        new URL(response.url()).pathname,
      ),
  );
  await page.getByRole("button", { name: "Save Configuration" }).click();
  const saveResponse = await saveResponsePromise;
  expect(saveResponse.ok(), await saveResponse.text()).toBeTruthy();
  const match = new URL(saveResponse.url()).pathname.match(
    /\/sessions\/([^/]+)\/agent-config$/,
  );
  expect(match).not.toBeNull();
  const sessionId = decodeURIComponent(match![1]);

  const saved = await request.get(
    `/api/uar/sessions/${encodeURIComponent(sessionId)}/agent-config`,
  );
  expect(saved.ok(), await saved.text()).toBeTruthy();
  await expect(saved.json()).resolves.toMatchObject({
    prompt_caching_enabled: true,
  });
  const effective = await request.get(
    `/api/uar/sessions/${encodeURIComponent(sessionId)}/prompt-caching`,
  );
  expect(effective.ok(), await effective.text()).toBeTruthy();
  await expect(effective.json()).resolves.toMatchObject({
    enabled: true,
    source: "session",
    session_override: true,
    global_default: false,
  });

  await configButton.click();
  await expect(promptCaching).toContainText("On");
  await expect(
    page.getByRole("status").filter({ hasText: "Effective now" }),
  ).toContainText("Effective now: On from session override");

  expect(
    observed.filter(
      (entry) =>
        entry.status === 404 &&
        /\/(agent-config|prompt-caching)$/.test(entry.path),
    ),
  ).toEqual([]);
  expect(observed.some((entry) => entry.status === 204)).toBeTruthy();
  expect(consoleErrors).toEqual([]);
  expect(pageErrors).toEqual([]);
});
