import { createBdd } from 'playwright-bdd';
import { expect, test, sendMessageAndWait } from '../support/world';
import { sendAndCapture } from '../support/s1-ui';

const { When, Then } = createBdd(test);

// Outside an S1 scenario `sendAndCapture` is exactly `sendMessageAndWait`; during
// one it also records the reply text the transcript showed for the message.
When('I send the message {string}', async ({ page, s1 }, message: string) => {
  await sendAndCapture(page, s1, message);
});

When('I ask {string}', async ({ page }, message: string) => {
  await sendMessageAndWait(page, message);
});

Then('the assistant responds with content containing {string}', async ({ page }, needle: string) => {
  await expect(page.getByText(needle, { exact: false }).last()).toBeVisible({ timeout: 15_000 });
});
