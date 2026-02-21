/**
 * Chat Input Bar Web Component
 *
 * A Cherry Studio-inspired chat input bar with:
 * - Auto-resizing textarea (min 2 rows, max 10 rows)
 * - Left toolbar slot for tool pills (agent selector, memory indicator, etc.)
 * - Right toolbar with token count badge and send/stop button
 * - File drag-and-drop zone
 * - Keyboard shortcuts: Enter = send, Shift+Enter = newline, Escape = clear
 *
 * Wraps the existing <file-upload> component for attachment handling.
 *
 * Usage:
 * ```html
 * <chat-input-bar
 *   placeholder="Message the agent…"
 *   session-id="sess-abc"
 *   agent-id="agent-xyz"
 * >
 *   <span slot="tools-left">
 *     <agent-selector></agent-selector>
 *     <memory-indicator></memory-indicator>
 *   </span>
 * </chat-input-bar>
 * ```
 *
 * Emits:
 * - `chat-send`  — { text: string, files: File[], agentId: string | null, sessionId: string | null }
 * - `chat-stop`  — {} (user clicked stop during streaming)
 */

import "../agent-selector/agent-selector";
import "../memory-indicator/memory-indicator";

export interface ChatSendEvent {
  text: string;
  files: File[];
  agentId: string | null;
  sessionId: string | null;
}

export class ChatInputBar extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["placeholder", "session-id", "agent-id", "streaming", "disabled"];
  }

  private textarea: HTMLTextAreaElement | null = null;
  private sendBtn: HTMLButtonElement | null = null;
  private tokenBadge: HTMLElement | null = null;
  private dragOverlay: HTMLElement | null = null;
  private attachedFiles: File[] = [];

  get placeholder(): string {
    return this.getAttribute("placeholder") ?? "Message the agent…";
  }
  get sessionId(): string | null {
    return this.getAttribute("session-id");
  }
  get agentId(): string | null {
    return this.getAttribute("agent-id");
  }
  get isStreaming(): boolean {
    return this.hasAttribute("streaming");
  }
  get isDisabled(): boolean {
    return this.hasAttribute("disabled");
  }

  connectedCallback(): void {
    this.render();
    this.bindEvents();
  }

  attributeChangedCallback(name: string): void {
    if (name === "streaming" || name === "disabled") {
      this.updateSendButton();
    }
  }

  private render(): void {
    this.innerHTML = `
      <style>
        chat-input-bar {
          display: flex;
          flex-direction: column;
          width: 100%;
        }
        .cib-container {
          display: flex;
          flex-direction: column;
          background: var(--color-surface-container, #1e2433);
          border: 1px solid var(--color-border, rgba(255,255,255,0.1));
          border-radius: 16px;
          transition: border-color 0.15s;
          overflow: hidden;
          position: relative;
        }
        .cib-container:focus-within {
          border-color: var(--color-primary, #6366f1);
        }
        .cib-container.drag-over {
          border-color: var(--color-primary, #6366f1);
          background: color-mix(in srgb, var(--color-primary, #6366f1) 8%, var(--color-surface-container, #1e2433));
        }
        .cib-textarea {
          width: 100%;
          background: transparent;
          border: none;
          outline: none;
          resize: none;
          padding: 14px 16px 0;
          font-size: 15px;
          line-height: 1.6;
          color: var(--color-text, #e2e8f0);
          font-family: inherit;
          min-height: 56px;
          max-height: 260px;
          overflow-y: auto;
          scrollbar-width: thin;
        }
        .cib-textarea::placeholder {
          color: var(--color-text-muted, #6b7280);
        }
        .cib-attachment-preview {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          padding: 8px 16px 0;
        }
        .cib-attachment-chip {
          display: inline-flex;
          align-items: center;
          gap: 6px;
          padding: 4px 10px;
          background: var(--color-surface, #141926);
          border: 1px solid var(--color-border, rgba(255,255,255,0.1));
          border-radius: 8px;
          font-size: 12px;
          color: var(--color-text-muted, #9ca3af);
        }
        .cib-attachment-chip button {
          background: none;
          border: none;
          cursor: pointer;
          color: var(--color-text-muted, #9ca3af);
          font-size: 14px;
          line-height: 1;
          padding: 0;
        }
        .cib-toolbar {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 8px 12px;
        }
        .cib-tools-left {
          display: flex;
          align-items: center;
          gap: 6px;
        }
        .cib-tools-right {
          display: flex;
          align-items: center;
          gap: 8px;
        }
        .cib-token-badge {
          font-size: 11px;
          color: var(--color-text-muted, #6b7280);
          font-variant-numeric: tabular-nums;
        }
        .cib-send-btn {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          width: 36px;
          height: 36px;
          border-radius: 10px;
          border: none;
          cursor: pointer;
          transition: background 0.15s, transform 0.1s;
          flex-shrink: 0;
        }
        .cib-send-btn:active { transform: scale(0.94); }
        .cib-send-btn.can-send {
          background: var(--color-primary, #6366f1);
          color: white;
        }
        .cib-send-btn.cannot-send {
          background: var(--color-border, rgba(255,255,255,0.08));
          color: var(--color-text-muted, #6b7280);
          cursor: not-allowed;
        }
        .cib-send-btn.streaming {
          background: var(--color-error, #ef4444);
          color: white;
        }
        .cib-attach-btn {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          width: 30px;
          height: 30px;
          border-radius: 8px;
          border: none;
          background: transparent;
          color: var(--color-text-muted, #6b7280);
          cursor: pointer;
          transition: color 0.15s, background 0.15s;
        }
        .cib-attach-btn:hover {
          color: var(--color-text, #e2e8f0);
          background: var(--color-border, rgba(255,255,255,0.08));
        }
        .cib-drag-overlay {
          display: none;
          position: absolute;
          inset: 0;
          border-radius: 16px;
          background: color-mix(in srgb, var(--color-primary, #6366f1) 15%, transparent);
          border: 2px dashed var(--color-primary, #6366f1);
          pointer-events: none;
          align-items: center;
          justify-content: center;
          font-size: 14px;
          font-weight: 600;
          color: var(--color-primary, #6366f1);
          z-index: 10;
        }
        .cib-drag-overlay.visible { display: flex; }
      </style>

      <div class="cib-container" id="cib-container">
        <div class="cib-drag-overlay" id="cib-drag-overlay">Drop files here</div>
        <div class="cib-attachment-preview" id="cib-attachment-preview"></div>
        <textarea
          class="cib-textarea"
          id="cib-textarea"
          placeholder="${this.placeholder}"
          rows="2"
          aria-label="Message input"
        ></textarea>
        <div class="cib-toolbar">
          <div class="cib-tools-left">
            <slot name="tools-left">
              <agent-selector
                ${this.agentId ? `agent-id="${this.agentId}"` : ""}
              ></agent-selector>
              <memory-indicator
                ${this.sessionId ? `session-id="${this.sessionId}"` : ""}
                ${this.agentId ? `agent-id="${this.agentId}"` : ""}
              ></memory-indicator>
            </slot>
            <button class="cib-attach-btn" id="cib-attach-btn" title="Attach file" type="button" aria-label="Attach file">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"/>
              </svg>
            </button>
            <input type="file" id="cib-file-input" multiple style="display:none" />
          </div>
          <div class="cib-tools-right">
            <span class="cib-token-badge" id="cib-token-badge"></span>
            <button class="cib-send-btn cannot-send" id="cib-send-btn" type="button" aria-label="Send message">
              ${this.sendIcon()}
            </button>
          </div>
        </div>
      </div>
    `;

    this.textarea = this.querySelector("#cib-textarea");
    this.sendBtn = this.querySelector("#cib-send-btn");
    this.tokenBadge = this.querySelector("#cib-token-badge");
    this.dragOverlay = this.querySelector("#cib-drag-overlay");
  }

  private sendIcon(): string {
    if (this.isStreaming) {
      return `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="2"/></svg>`;
    }
    return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>`;
  }

  private bindEvents(): void {
    const container = this.querySelector<HTMLElement>("#cib-container");
    const fileInput = this.querySelector<HTMLInputElement>("#cib-file-input");
    const attachBtn = this.querySelector<HTMLButtonElement>("#cib-attach-btn");

    // Textarea auto-resize + token estimate
    this.textarea?.addEventListener("input", () => {
      this.autoResize();
      this.updateTokenEstimate();
      this.updateSendButton();
    });

    // Keyboard shortcuts
    this.textarea?.addEventListener("keydown", (e: KeyboardEvent) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        this.handleSend();
      }
      if (e.key === "Escape") {
        this.clearInput();
      }
    });

    // Send / stop button
    this.sendBtn?.addEventListener("click", () => {
      if (this.isStreaming) {
        this.dispatchEvent(new CustomEvent("chat-stop", { bubbles: true, composed: true }));
      } else {
        this.handleSend();
      }
    });

    // File attachment via button
    attachBtn?.addEventListener("click", () => fileInput?.click());
    fileInput?.addEventListener("change", (e) => {
      const files = (e.target as HTMLInputElement).files;
      if (files) this.addFiles(Array.from(files));
    });

    // Drag-and-drop
    container?.addEventListener("dragenter", (e) => {
      e.preventDefault();
      container.classList.add("drag-over");
      this.dragOverlay?.classList.add("visible");
    });
    container?.addEventListener("dragleave", (e) => {
      if (!container.contains(e.relatedTarget as Node)) {
        container.classList.remove("drag-over");
        this.dragOverlay?.classList.remove("visible");
      }
    });
    container?.addEventListener("dragover", (e) => e.preventDefault());
    container?.addEventListener("drop", (e) => {
      e.preventDefault();
      container.classList.remove("drag-over");
      this.dragOverlay?.classList.remove("visible");
      const files = e.dataTransfer?.files;
      if (files) this.addFiles(Array.from(files));
    });

    // Paste with files
    this.textarea?.addEventListener("paste", (e: ClipboardEvent) => {
      const items = e.clipboardData?.items;
      if (!items) return;
      const fileItems = Array.from(items).filter((i) => i.kind === "file");
      if (fileItems.length) {
        const files = fileItems.map((i) => i.getAsFile()).filter(Boolean) as File[];
        this.addFiles(files);
      }
    });
  }

  private autoResize(): void {
    if (!this.textarea) return;
    this.textarea.style.height = "auto";
    const lineHeight = 24;
    const minHeight = lineHeight * 2;
    const maxHeight = lineHeight * 10;
    const scrollHeight = this.textarea.scrollHeight;
    this.textarea.style.height = `${Math.min(Math.max(scrollHeight, minHeight), maxHeight)}px`;
  }

  private updateTokenEstimate(): void {
    if (!this.tokenBadge || !this.textarea) return;
    const text = this.textarea.value;
    // Rough estimate: 1 token ≈ 4 chars
    const estimate = Math.ceil(text.length / 4);
    this.tokenBadge.textContent = estimate > 0 ? `~${estimate} tok` : "";
  }

  private updateSendButton(): void {
    if (!this.sendBtn || !this.textarea) return;
    const hasContent = this.textarea.value.trim().length > 0 || this.attachedFiles.length > 0;

    if (this.isStreaming) {
      this.sendBtn.className = "cib-send-btn streaming";
      this.sendBtn.innerHTML = `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="2"/></svg>`;
      this.sendBtn.disabled = false;
    } else if (hasContent && !this.isDisabled) {
      this.sendBtn.className = "cib-send-btn can-send";
      this.sendBtn.innerHTML = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>`;
      this.sendBtn.disabled = false;
    } else {
      this.sendBtn.className = "cib-send-btn cannot-send";
      this.sendBtn.disabled = true;
    }
  }

  private handleSend(): void {
    if (!this.textarea) return;
    const text = this.textarea.value.trim();
    if (!text && this.attachedFiles.length === 0) return;
    if (this.isDisabled) return;

    const detail: ChatSendEvent = {
      text,
      files: [...this.attachedFiles],
      agentId: this.agentId,
      sessionId: this.sessionId,
    };

    this.dispatchEvent(
      new CustomEvent<ChatSendEvent>("chat-send", {
        bubbles: true,
        composed: true,
        detail,
      }),
    );

    this.clearInput();
  }

  private clearInput(): void {
    if (this.textarea) {
      this.textarea.value = "";
      this.textarea.style.height = "";
    }
    this.attachedFiles = [];
    this.renderAttachmentPreview();
    this.updateSendButton();
    if (this.tokenBadge) this.tokenBadge.textContent = "";
  }

  private addFiles(files: File[]): void {
    this.attachedFiles.push(...files);
    this.renderAttachmentPreview();
    this.updateSendButton();
  }

  private renderAttachmentPreview(): void {
    const preview = this.querySelector("#cib-attachment-preview");
    if (!preview) return;
    preview.innerHTML = this.attachedFiles
      .map(
        (f, i) => `
        <div class="cib-attachment-chip">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
          <span>${f.name}</span>
          <button data-index="${i}" aria-label="Remove ${f.name}">×</button>
        </div>`,
      )
      .join("");

    preview.querySelectorAll("button[data-index]").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const idx = parseInt((e.target as HTMLElement).getAttribute("data-index") ?? "0", 10);
        this.attachedFiles.splice(idx, 1);
        this.renderAttachmentPreview();
        this.updateSendButton();
      });
    });
  }

  /** Programmatically put the input bar into streaming mode. */
  public setStreaming(streaming: boolean): void {
    if (streaming) {
      this.setAttribute("streaming", "");
    } else {
      this.removeAttribute("streaming");
    }
    this.updateSendButton();
  }

  /** Focus the textarea. */
  public focus(): void {
    this.textarea?.focus();
  }
}

customElements.define("chat-input-bar", ChatInputBar);
