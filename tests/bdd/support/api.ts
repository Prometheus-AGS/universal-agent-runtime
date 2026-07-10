import { APP_BASE_URL, STUB_BASE_URL } from './ports';

async function fetchAllRuntimeAgents(): Promise<Record<string, unknown>[]> {
  const res = await fetch(`${APP_BASE_URL}/api/agents`);
  if (!res.ok) throw new Error(`GET /api/agents failed: ${res.status}`);
  const body = (await res.json()) as { runtime_agents: Record<string, unknown>[] };
  return body.runtime_agents;
}

/** Fetches the first runtime agent from the catalog, used as a clone template for test agents. */
async function fetchTemplateAgent(): Promise<Record<string, unknown>> {
  const agents = await fetchAllRuntimeAgents();
  const template = agents[0];
  if (!template) throw new Error('no runtime agent available to clone as a BDD test-agent template');
  return template;
}

/**
 * Deletes any existing agent whose title matches exactly. The full BDD suite
 * shares one ephemeral SurrealKV store across all 6 scenarios in a run (see
 * playwright.config.ts) — without this, re-running a scenario with the same
 * agent title (or two scenarios sharing a title, e.g. "BDD Agent Aurora" in
 * both chat-agent-switching and chat-model-routing) leaves duplicate
 * same-titled agents behind, and `switchAgentViaUI`'s title-text click
 * becomes ambiguous (picks whichever duplicate sorts first, not necessarily
 * this scenario's own freshly-configured one).
 */
async function deleteAgentsByTitle(title: string): Promise<void> {
  const agents = await fetchAllRuntimeAgents();
  const matches = agents.filter((a) => (a.metadata as Record<string, unknown> | undefined)?.title === title);
  for (const agent of matches) {
    await fetch(`${APP_BASE_URL}/api/agents/${agent.id}`, { method: 'DELETE' });
  }
}

export interface TestAgentOverrides {
  title: string;
  model?: string; // "{provider}/{model}", overrides policy.provider.default
  knowledgeBases?: string[];
  skillIds?: string[];
  tools?: string[];
}

/**
 * Creates a fresh test agent by cloning the first catalog agent and applying
 * overrides, via the real `POST /api/agents` endpoint — avoids depending on
 * whatever knowledge bases / skills the ambient default agent happens to have.
 */
export async function createTestAgent(overrides: TestAgentOverrides): Promise<{ id: string; model: string }> {
  await deleteAgentsByTitle(overrides.title);
  const template = structuredClone(await fetchTemplateAgent());
  const id = crypto.randomUUID();
  template.id = id;
  (template.metadata as Record<string, unknown>).title = overrides.title;

  const policy = template.policy as Record<string, unknown>;
  const provider = (policy.provider as Record<string, unknown>).default as Record<string, unknown>;
  // Always set explicitly — the cloned template's own default (e.g. the
  // seeded default-agent's "gpt-5.2") isn't a model the stub-llm fixtures
  // are keyed on or that this test env's single registered custom provider
  // recognizes, so an agent that didn't ask for a specific model would
  // otherwise get a real "model_not_found" error instead of ever reaching
  // the stub.
  const [providerId, modelId] = (overrides.model ?? 'openai/gpt-5.4-mini').split('/');
  provider.provider = providerId;
  provider.model = modelId;
  policy.skills = { ...(policy.skills as object), prefer: overrides.skillIds ?? [] };
  policy.tools = { ...(policy.tools as object), allow: overrides.tools ?? [] };

  const memory = template.memory as Record<string, unknown>;
  (memory.kb as Record<string, unknown>).enabled = (overrides.knowledgeBases?.length ?? 0) > 0;
  (memory.kb as Record<string, unknown>).knowledge_bases = overrides.knowledgeBases ?? [];

  const res = await fetch(`${APP_BASE_URL}/api/agents`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(template),
  });
  if (!res.ok) throw new Error(`POST /api/agents failed: ${res.status} ${await res.text()}`);
  const created = (await res.json()) as Record<string, unknown>;
  const createdProvider = ((created.policy as Record<string, unknown>).provider as Record<string, unknown>)
    .default as Record<string, unknown>;
  return { id, model: `${createdProvider.provider}/${createdProvider.model}` };
}

/** Deletes an existing knowledge base by exact name, if one exists (name uniqueness is enforced server-side). */
async function deleteKnowledgeBaseByName(name: string): Promise<void> {
  const res = await fetch(`${APP_BASE_URL}/api/knowledge`);
  if (!res.ok) return;
  const list = (await res.json()) as Array<{ id: string; name: string }>;
  const match = list.find((kb) => kb.name === name);
  if (match) await fetch(`${APP_BASE_URL}/api/knowledge/${match.id}`, { method: 'DELETE' });
}

/**
 * Registers a second custom LLM provider pointed at the same stub-llm base
 * URL, distinct from the one custom provider `tests/bdd/playwright.config.ts`
 * registers via `UAR_LLM__BASE_URL`/`UAR_LLM__MODEL` env vars (which only
 * recognizes one model id, `openai/gpt-5.4-mini` — any other model string
 * under that same provider id fails real model-catalog validation with
 * "model_not_found", confirmed empirically). Registering a distinct provider
 * id is how scenario 6 (provider/model routing) gets a second, independently
 * addressable model without touching the env-var-configured default.
 * Idempotent — ignores 409 (already registered) from a prior scenario run
 * sharing this suite invocation's single ephemeral server.
 */
export async function ensureSecondTestProvider(): Promise<void> {
  const res = await fetch(`${APP_BASE_URL}/api/uar/providers`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      id: 'bdd-provider-b',
      display_name: 'BDD Test Provider B',
      base_url: `${STUB_BASE_URL}/v1`,
      default_model: 'claude-5-haiku',
      models: [{ id: 'claude-5-haiku' }],
      enabled: true,
    }),
  });
  if (!res.ok && res.status !== 409) {
    throw new Error(`POST /api/uar/providers failed: ${res.status} ${await res.text()}`);
  }
}

/** Creates a knowledge base and uploads a single text document containing `content`. */
export async function createKnowledgeBaseWithDocument(name: string, content: string): Promise<string> {
  await deleteKnowledgeBaseByName(name);
  const kbRes = await fetch(`${APP_BASE_URL}/api/knowledge`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, description: `BDD fixture KB: ${name}` }),
  });
  if (!kbRes.ok) throw new Error(`POST /api/knowledge failed: ${kbRes.status} ${await kbRes.text()}`);
  const kb = (await kbRes.json()) as { id: string };

  const form = new FormData();
  form.append('file', new Blob([content], { type: 'text/plain' }), 'bdd-fixture.txt');
  const uploadRes = await fetch(`${APP_BASE_URL}/api/knowledge/${kb.id}/documents`, {
    method: 'POST',
    body: form,
  });
  if (!uploadRes.ok) throw new Error(`document upload failed: ${uploadRes.status} ${await uploadRes.text()}`);
  const doc = (await uploadRes.json()) as { id: string };

  // Ingestion is async (submitted to a worker pool, 202 Accepted) — poll
  // until it reaches a terminal status before returning, so callers don't
  // race the query against not-yet-indexed content.
  const deadline = Date.now() + 30_000;
  for (;;) {
    const statusRes = await fetch(`${APP_BASE_URL}/api/knowledge/${kb.id}/documents/${doc.id}`);
    if (statusRes.ok) {
      const status = (await statusRes.json()) as { status: string; error_message?: string | null };
      const s = status.status.toLowerCase();
      if (s === 'indexed') break;
      if (s === 'failed') {
        throw new Error(`document ingestion failed: ${status.error_message ?? '(no error_message)'}`);
      }
    }
    if (Date.now() > deadline) throw new Error(`document ingestion did not reach a terminal status within 30s`);
    await new Promise((resolve) => setTimeout(resolve, 250));
  }

  return kb.id;
}

/** Creates a skill with a keyword trigger, prompt overlay only (no mcp_config, no preferred_model). */
export async function createTestSkill(opts: {
  name: string;
  keyword: string;
  promptOverlay: string;
}): Promise<string> {
  const res = await fetch(`${APP_BASE_URL}/api/skills`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      name: opts.name,
      description: `BDD fixture skill: ${opts.name}`,
      triggers: { keywords: [opts.keyword], semantic: null },
      prompt_overlay: opts.promptOverlay,
      enabled: true,
    }),
  });
  if (!res.ok) throw new Error(`POST /api/skills failed: ${res.status} ${await res.text()}`);
  const skill = (await res.json()) as { skill_id: string };
  return skill.skill_id;
}

/** Binds a skill to an agent via PUT /api/agents/{agentId}/skills. */
export async function bindSkillToAgent(agentId: string, skillId: string): Promise<void> {
  const res = await fetch(`${APP_BASE_URL}/api/agents/${agentId}/skills`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ skill_ids: [skillId] }),
  });
  if (!res.ok) throw new Error(`PUT /api/agents/${agentId}/skills failed: ${res.status} ${await res.text()}`);
}

