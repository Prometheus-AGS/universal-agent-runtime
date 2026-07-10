import { createBdd } from 'playwright-bdd';
import { expect, test, openFreshThread, switchAgentViaUI } from '../support/world';
import { createTestAgent, ensureSecondTestProvider } from '../support/api';

const { Given, When, Then } = createBdd(test);

Given(
  'two agents {string} \\(model {string}\\) and {string} \\(model {string}\\) exist',
  async ({}, titleA: string, modelA: string, titleB: string, modelB: string) => {
    await ensureSecondTestProvider();
    await createTestAgent({ title: titleA, model: modelA });
    await createTestAgent({ title: titleB, model: modelB });
  },
);

Given('a fresh conversation using {string}', async ({ page }, agentTitle: string) => {
  await openFreshThread(page);
  await switchAgentViaUI(page, agentTitle);
});

When('I switch the active agent to {string}', async ({ page }, agentTitle: string) => {
  await switchAgentViaUI(page, agentTitle);
});

Then("the outgoing request's model field is {string}", async ({ stub }, model: string) => {
  const requests = await stub.requests();
  const last = requests.at(-1);
  expect(last, 'expected at least one captured stub-LLM request').toBeTruthy();
  // Observed empirically (stub-llm's /_stub/requests introspection) that the
  // wire-level model field is sometimes the bare model id and sometimes
  // "provider/model", depending on which resolution path a given run takes
  // (registry.rs's has_explicit_base_url branch vs. a global-config fallback)
  // — an implementation-internal formatting detail, not what this scenario
  // is proving. Accepting either form the bare id can take keeps the
  // assertion about *routing* (did the right model answer) without being
  // fragile to that formatting variance.
  expect(last!.model.endsWith(model)).toBe(true);
});
