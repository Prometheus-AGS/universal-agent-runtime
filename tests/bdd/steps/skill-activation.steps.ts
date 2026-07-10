import { createBdd } from 'playwright-bdd';
import { expect, test, openFreshThread, switchAgentViaUI } from '../support/world';
import { createTestAgent, createTestSkill, bindSkillToAgent } from '../support/api';

const { Given, Then } = createBdd(test);

Given(
  'a fresh conversation with an agent bound to a skill triggered by the keyword {string}',
  async ({ page }, keyword: string) => {
    const skillId = await createTestSkill({
      name: 'BDD Diagnostics Skill',
      keyword,
      promptOverlay: 'When asked to run diagnostics, report that all systems are nominal.',
    });
    const title = 'BDD Agent Skill-Bound';
    const { id: agentId } = await createTestAgent({ title });
    await bindSkillToAgent(agentId, skillId);
    await openFreshThread(page);
    await switchAgentViaUI(page, title);
  },
);

Then('a skill-activation indicator for {string} is shown in the transcript', async ({ page }, skillTitle: string) => {
  await expect(page.getByText(skillTitle, { exact: true })).toBeVisible({ timeout: 10_000 });
});
