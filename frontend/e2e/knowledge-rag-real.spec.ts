import { expect, test } from "@chromatic-com/playwright";

test("knowledge page creates, indexes, searches, and deletes real data", async ({ page }) => {
  const baseName = `Release facts ${Date.now()}`;
  const filename = "release-fact.txt";
  const fact = "The UAR production certification marker is COBALT-7319.";

  await page.goto("/admin/knowledge");
  await page.getByRole("button", { name: "New", exact: true }).click();
  await page.getByLabel("Name", { exact: true }).fill(baseName);
  await page.getByLabel("Description", { exact: true }).fill("Real RAG browser certification");
  await page.getByRole("button", { name: "Create", exact: true }).click();

  const baseLabel = page.getByText(baseName, { exact: true });
  await expect(baseLabel).toBeVisible();
  await baseLabel.click();

  await page.getByLabel("Select files to upload to this knowledge base").setInputFiles({
    name: filename,
    mimeType: "text/plain",
    buffer: Buffer.from(fact),
  });

  const documentRow = page.getByTestId(/^knowledge-document-/).filter({ hasText: filename });
  await expect(documentRow).toBeVisible();
  await expect(documentRow.getByText("indexed", { exact: true })).toBeVisible({
    timeout: 30_000,
  });

  await page.getByPlaceholder("Search by meaning across all documents...").fill(
    "production certification marker",
  );
  await page.getByRole("button", { name: "Search", exact: true }).click();
  await expect(page.getByText(fact, { exact: true })).toBeVisible({ timeout: 15_000 });
  await expect(page.getByText(/^Score: /)).toBeVisible();

  await page.getByRole("button", { name: "Clear", exact: true }).click();
  await page.getByRole("button", { name: `Delete ${filename}` }).click();
  await page.getByRole("alertdialog").getByRole("button", { name: "Delete" }).click();
  await expect(page.getByText(filename, { exact: true })).not.toBeVisible();

  await page.getByRole("button", { name: "Back to knowledge bases" }).click();
  await page.locator(`button[aria-label="Delete ${baseName}"]`).click();
  await page.getByRole("alertdialog").getByRole("button", { name: "Delete" }).click();
  await expect(page.getByText(baseName, { exact: true })).not.toBeVisible();
});
