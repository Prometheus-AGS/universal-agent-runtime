/**
 * The one rule for "this agent keeps its own model": the runtime's `explicit_model` on the agents
 * read (`provider/model`, or null). The runtime derives it from the same artifact and extension
 * fields it resolves a turn with, so the console does not re-implement that rule. A federated
 * agent runs elsewhere and is never counted.
 */

export interface ExplicitModelSource {
  explicit_model?: unknown;
  _type?: unknown;
}

/** The agent's own `provider/model`, or null when it follows the system default. */
export function explicitAgentModel(agent: ExplicitModelSource | null | undefined): string | null {
  if (!agent || agent._type === "federated") return null;
  const model = agent.explicit_model;
  return typeof model === "string" && model.trim() !== "" ? model : null;
}

/** True for a runtime agent that the system default model does not reach. */
export function isPinnedAgent(agent: ExplicitModelSource | null | undefined): boolean {
  return explicitAgentModel(agent) !== null;
}
