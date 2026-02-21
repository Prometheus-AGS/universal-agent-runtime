/**
 * Agent Selector Web Component
 *
 * A pill button in the chat input bar's left toolbar that displays the
 * currently selected agent and provides a dropdown to switch agents.
 *
 * - Fetches agent list from GET /api/uar/agents on connect
 * - Shows a dropdown of available agents on click
 * - Passes `agent-id` attribute to the chat-input-bar and run requests
 * - Auto-selects the first agent if none is pre-selected
 *
 * Usage:
 * ```html
 * <agent-selector agent-id="agent-abc"></agent-selector>
 * ```
 *
 * Emits:
 * - `agent-selected` — { agentId: string, agentTitle: string }
 */

interface AgentSummary {
  id: string;
  title: string;
  description?: string;
  version?: string;
  kind?: string;
}

export class AgentSelector extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["agent-id"];
  }

  private agents: AgentSummary[] = [];
  private selectedAgent: AgentSummary | null = null;
  private dropdownOpen = false;
  private loading = false;

  get agentId(): string | null {
    return this.getAttribute("agent-id");
  }

  connectedCallback(): void {
    this.render();
    void this.fetchAgents();
    document.addEventListener("click", this.handleOutsideClick);
  }

  disconnectedCallback(): void {
    document.removeEventListener("click", this.handleOutsideClick);
  }

  attributeChangedCallback(name: string, _old: string, val: string): void {
    if (name === "agent-id" && val && this.agents.length > 0) {
      this.selectedAgent = this.agents.find((a) => a.id === val) ?? this.selectedAgent;
      this.render();
    }
  }

  private handleOutsideClick = (e: MouseEvent): void => {
    if (!this.contains(e.target as Node) && this.dropdownOpen) {
      this.dropdownOpen = false;
      this.render();
    }
  };

  private async fetchAgents(): Promise<void> {
    this.loading = true;
    this.render();
    try {
      const res = await fetch("/api/uar/agents");
      if (!res.ok) return;
      // API returns agents with the pattern used in server.rs discovery endpoint
      const raw = (await res.json()) as { agents?: AgentSummary[] } | AgentSummary[];
      const list = Array.isArray(raw) ? raw : (raw.agents ?? []);

      // Normalize to AgentSummary shape
      this.agents = list.map((a) => ({
        id: a.id,
        title: a.title ?? a.id,
        description: a.description,
        version: a.version,
        kind: a.kind,
      }));

      // Auto-select: prefer pre-set agent-id, else first in list
      const preSet = this.agentId;
      if (preSet && this.agents.length > 0) {
        this.selectedAgent = this.agents.find((a) => a.id === preSet) ?? this.agents[0];
      } else if (this.agents.length > 0) {
        this.selectedAgent = this.agents[0];
      }

      if (this.selectedAgent) {
        this.setAttribute("agent-id", this.selectedAgent.id);
        this.emitSelection(this.selectedAgent);
      }
    } catch {
      // Agents endpoint may not be available — show placeholder
    } finally {
      this.loading = false;
      this.render();
    }
  }

  private emitSelection(agent: AgentSummary): void {
    this.dispatchEvent(
      new CustomEvent("agent-selected", {
        bubbles: true,
        composed: true,
        detail: { agentId: agent.id, agentTitle: agent.title },
      }),
    );
  }

  private handleToggle(): void {
    if (this.agents.length <= 1) return;
    this.dropdownOpen = !this.dropdownOpen;
    this.render();
  }

  private handleSelect(agent: AgentSummary): void {
    this.selectedAgent = agent;
    this.dropdownOpen = false;
    this.setAttribute("agent-id", agent.id);
    this.emitSelection(agent);
    this.render();
  }

  private render(): void {
    const label = this.loading
      ? "Loading…"
      : this.selectedAgent
        ? this.selectedAgent.title
        : this.agents.length === 0
          ? "No agents"
          : "Select agent";

    const hasMultiple = this.agents.length > 1;

    this.innerHTML = `
      <style>
        agent-selector {
          display: inline-flex;
          align-items: center;
          position: relative;
        }
        .as-pill {
          display: inline-flex;
          align-items: center;
          gap: 5px;
          padding: 3px 10px;
          border-radius: 999px;
          border: 1px solid var(--color-border, rgba(255,255,255,0.15));
          background: transparent;
          cursor: ${hasMultiple ? "pointer" : "default"};
          font-size: 12px;
          font-weight: 500;
          color: var(--color-text, #e2e8f0);
          transition: border-color 0.15s, background 0.15s;
          max-width: 160px;
          white-space: nowrap;
          overflow: hidden;
        }
        .as-pill:hover {
          border-color: ${hasMultiple ? "var(--color-primary, #6366f1)" : "var(--color-border, rgba(255,255,255,0.15))"};
          background: ${hasMultiple ? "color-mix(in srgb, var(--color-primary, #6366f1) 8%, transparent)" : "transparent"};
        }
        .as-icon {
          width: 12px;
          height: 12px;
          border-radius: 50%;
          background: var(--color-primary, #6366f1);
          flex-shrink: 0;
        }
        .as-label {
          overflow: hidden;
          text-overflow: ellipsis;
        }
        .as-chevron {
          opacity: 0.5;
          flex-shrink: 0;
        }
        .as-dropdown {
          position: absolute;
          bottom: calc(100% + 8px);
          left: 0;
          min-width: 240px;
          max-width: 320px;
          background: var(--color-surface, #141926);
          border: 1px solid var(--color-border, rgba(255,255,255,0.12));
          border-radius: 12px;
          box-shadow: 0 8px 32px rgba(0,0,0,0.4);
          overflow: hidden;
          z-index: 100;
        }
        .as-dropdown-header {
          padding: 10px 14px;
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.06em;
          color: var(--color-text-muted, #6b7280);
          border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.08));
        }
        .as-agent-item {
          display: flex;
          flex-direction: column;
          gap: 2px;
          padding: 10px 14px;
          cursor: pointer;
          transition: background 0.1s;
        }
        .as-agent-item:hover {
          background: var(--color-surface-container, #1e2433);
        }
        .as-agent-item.selected {
          background: color-mix(in srgb, var(--color-primary, #6366f1) 10%, var(--color-surface, #141926));
        }
        .as-agent-name {
          font-size: 13px;
          font-weight: 500;
          color: var(--color-text, #e2e8f0);
        }
        .as-agent-desc {
          font-size: 11px;
          color: var(--color-text-muted, #6b7280);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }
      </style>

      <button
        class="as-pill"
        type="button"
        title="${this.selectedAgent?.description ?? label}"
        aria-label="Selected agent: ${label}"
        aria-haspopup="${hasMultiple}"
        aria-expanded="${this.dropdownOpen}"
        id="as-toggle"
      >
        <span class="as-icon"></span>
        <span class="as-label">${label}</span>
        ${hasMultiple ? `<svg class="as-chevron" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>` : ""}
      </button>

      ${
        this.dropdownOpen && hasMultiple
          ? `<div class="as-dropdown" role="listbox" aria-label="Select an agent">
              <div class="as-dropdown-header">Select Agent</div>
              ${this.agents
                .map(
                  (a) => `
                <div class="as-agent-item ${a.id === this.selectedAgent?.id ? "selected" : ""}"
                  role="option"
                  aria-selected="${a.id === this.selectedAgent?.id}"
                  data-agent-id="${a.id}"
                >
                  <span class="as-agent-name">${a.title}</span>
                  ${a.description ? `<span class="as-agent-desc">${a.description}</span>` : ""}
                </div>`,
                )
                .join("")}
            </div>`
          : ""
      }
    `;

    this.querySelector("#as-toggle")?.addEventListener("click", (e) => {
      e.stopPropagation();
      this.handleToggle();
    });

    this.querySelectorAll(".as-agent-item[data-agent-id]").forEach((item) => {
      item.addEventListener("click", (e) => {
        e.stopPropagation();
        const id = (e.currentTarget as HTMLElement).getAttribute("data-agent-id") ?? "";
        const agent = this.agents.find((a) => a.id === id);
        if (agent) this.handleSelect(agent);
      });
    });
  }
}

customElements.define("agent-selector", AgentSelector);
