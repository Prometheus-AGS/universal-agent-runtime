import { createBdd } from 'playwright-bdd';
import { expect, test, openFreshThread, switchAgentViaUI, systemPromptOf } from '../support/world';
import { createTestAgent, createKnowledgeBaseWithDocument } from '../support/api';

const { Given, Then } = createBdd(test);

let lastKbId: string | undefined;

Given('I have ingested a knowledge base document containing {string}', async ({}, phrase: string) => {
  lastKbId = await createKnowledgeBaseWithDocument('bdd-kb-retrieval', phrase);
});

Given('a fresh conversation with an agent scoped to that knowledge base', async ({ page }) => {
  if (!lastKbId) throw new Error('no knowledge base id — did the ingestion step run first?');
  const title = 'BDD Agent KB-Scoped';
  await createTestAgent({ title, knowledgeBases: ['bdd-kb-retrieval'] });
  await openFreshThread(page);
  await switchAgentViaUI(page, title);
});

Then("the outgoing request's system prompt actually contains {string}", async ({ stub }, phrase: string) => {
  const requests = await stub.requests();
  const last = requests.at(-1);
  expect(last, 'expected at least one captured stub-LLM request').toBeTruthy();
  expect(systemPromptOf(last!)).toContain(phrase);
});
