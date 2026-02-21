/**
 * Memory Indicator Web Component
 *
 * A compact pill badge for the chat input bar that shows whether UAR's
 * memory system is active, the current scope, and the record count.
 *
 * - Click to cycle through scopes: session → user → agent → global → off
 * - Green when memory is active, grey when off
 * - Fetches stats from GET /api/uar/memory/stats every 30 seconds
 *
 * Usage:
 * ```html
 * <memory-indicator session-id="sess-abc" agent-id="agent-xyz"></memory-indicator>
 * ```
 *
 * Emits:
 * - `memory-scope-changed` — { scope: string | null } when the user changes scope
 */

const SCOPES = ["session", "user", "agent", "global"] as const;
type MemoryScope = (typeof SCOPES)[number] | null;

interface MemoryStats {
  enabled?: boolean;
  total: number;
  by_scope: Record<string, number>;
}

export class MemoryIndicator extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["session-id", "agent-id", "scope"];
  }

  private currentScope: MemoryScope = "session";
  private stats: MemoryStats | null = null;
  private pollTimer: ReturnType<typeof setInterval> | null = null;
  private loading = false;

  get sessionId(): string {
    return this.getAttribute("session-id") ?? "";
  }
  get agentId(): string {
    return this.getAttribute("agent-id") ?? "";
  }

  connectedCallback(): void {
    this.render();
    void this.fetchStats();
    // Poll every 30 seconds
    this.pollTimer = setInterval(() => {
      void this.fetchStats();
    }, 30_000);
    this.addEventListener("click", this.handleClick);
  }

  disconnectedCallback(): void {
    if (this.pollTimer !== null) clearInterval(this.pollTimer);
    this.removeEventListener("click", this.handleClick);
  }

  attributeChangedCallback(name: string, _old: string, _new: string): void {
    if (name === "scope") {
      this.currentScope = (_new as MemoryScope) ?? null;
    }
    if (this.isConnected) this.render();
  }

  private handleClick = (): void => {
    if (this.loading) return;
    this.cycleScope();
  };

  private cycleScope(): void {
    if (this.currentScope === null) {
      this.currentScope = SCOPES[0];
    } else {
      const idx = SCOPES.indexOf(this.currentScope);
      if (idx === SCOPES.length - 1) {
        this.currentScope = null; // off
      } else {
        this.currentScope = SCOPES[idx + 1];
      }
    }
    this.render();
    this.dispatchEvent(
      new CustomEvent("memory-scope-changed", {
        bubbles: true,
        composed: true,
        detail: { scope: this.currentScope },
      }),
    );
  }

  private async fetchStats(): Promise<void> {
    this.loading = true;
    try {
      const params = new URLSearchParams();
      if (this.sessionId) params.set("session_id", this.sessionId);
      if (this.agentId) params.set("agent_id", this.agentId);

      const res = await fetch(`/api/admin/memories/stats?${params.toString()}`);
      if (res.ok) {
        this.stats = (await res.json()) as MemoryStats;
      }
    } catch {
      // Server may not have memory enabled — silently ignore
    } finally {
      this.loading = false;
      this.render();
    }
  }

  private render(): void {
    const isOff = this.currentScope === null;
    const count = this.stats
      ? this.currentScope
        ? (this.stats.by_scope[this.currentScope] ?? 0)
        : 0
      : null;

    const label = isOff ? "MEM OFF" : `MEM · ${this.currentScope}`;
    const countBadge = count !== null && !isOff ? ` (${count})` : "";
    const title = isOff
      ? "Memory is off — click to enable"
      : `Memory active: ${this.currentScope} scope${count !== null ? `, ${count} records` : ""}. Click to cycle scope.`;

    this.innerHTML = `
      <button
        class="memory-indicator-pill ${isOff ? "memory-off" : "memory-on"}"
        title="${title}"
        aria-label="${title}"
        type="button"
      >
        <span class="memory-dot ${isOff ? "dot-off" : "dot-on"}"></span>
        <span class="memory-label">${label}${countBadge}</span>
      </button>
      <style>
        memory-indicator {
          display: inline-flex;
          align-items: center;
        }
        .memory-indicator-pill {
          display: inline-flex;
          align-items: center;
          gap: 5px;
          padding: 3px 10px;
          border-radius: 999px;
          border: 1px solid;
          cursor: pointer;
          font-size: 11px;
          font-weight: 600;
          letter-spacing: 0.04em;
          transition: background 0.15s, border-color 0.15s;
          user-select: none;
        }
        .memory-on {
          background: color-mix(in srgb, var(--color-success, #22c55e) 12%, transparent);
          border-color: color-mix(in srgb, var(--color-success, #22c55e) 40%, transparent);
          color: var(--color-success, #22c55e);
        }
        .memory-on:hover {
          background: color-mix(in srgb, var(--color-success, #22c55e) 20%, transparent);
        }
        .memory-off {
          background: transparent;
          border-color: var(--color-border, rgba(255,255,255,0.15));
          color: var(--color-text-muted, #6b7280);
        }
        .memory-off:hover {
          border-color: var(--color-border-hover, rgba(255,255,255,0.3));
          color: var(--color-text, #d1d5db);
        }
        .memory-dot {
          width: 6px;
          height: 6px;
          border-radius: 50%;
          flex-shrink: 0;
        }
        .dot-on {
          background: var(--color-success, #22c55e);
          box-shadow: 0 0 4px var(--color-success, #22c55e);
        }
        .dot-off {
          background: var(--color-text-muted, #6b7280);
        }
      </style>
    `;
  }
}

customElements.define("memory-indicator", MemoryIndicator);
