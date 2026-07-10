import { createBdd } from 'playwright-bdd';
import { expect, test, sendMessageAndWait } from '../support/world';

const { When, Then } = createBdd(test);

When('I send the message {string}', async ({ page }, message: string) => {
  await sendMessageAndWait(page, message);
});

When('I ask {string}', async ({ page }, message: string) => {
  await sendMessageAndWait(page, message);
});

Then('the assistant responds with content containing {string}', async ({ page }, needle: string) => {
  await expect(page.getByText(needle, { exact: false }).last()).toBeVisible({ timeout: 15_000 });
});
