/**
 * A2UI Artifact Web Component
 *
 * Renders interactive artifacts emitted by agents via `agui.artifact_input_request`
 * and `agui.artifact` SSE events. Supports four interactive types that pause the
 * agent and collect user input:
 *
 *   - form       — a structured form driven by a JSON Schema
 *   - confirm    — a yes/no or accept/cancel dialog
 *   - select     — a single-choice option list
 *   - text_input — a single free-text field
 *
 * And two display-only types that do not pause the agent:
 *
 *   - display — rendered markdown content
 *   - chart   — a chart or visualization (rendered as JSON for now)
 *
 * Usage (set by chat-stream when routing artifact events):
 *
 * ```html
 * <a2ui-artifact
 *   run-id="run-abc123"
 *   artifact-id="artifact-xyz"
 *   artifact-type="form"
 *   title="Tell us about yourself"
 *   content='{"properties":{"name":{"type":"string","title":"Your name"}}}'
 * ></a2ui-artifact>
 * ```
 */

interface ArtifactFormField {
  type: string;
  title?: string;
  description?: string;
  enum?: string[];
  items?: { type: string };
  minimum?: number;
  maximum?: number;
}

interface ArtifactFormSchema {
  type?: string;
  title?: string;
  description?: string;
  required?: string[];
  properties?: Record<string, ArtifactFormField>;
}

type ArtifactSubmitResult =
  | { accepted: boolean } // confirm
  | { value: string } // select, text_input
  | Record<string, unknown>; // form

export class A2uiArtifact extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["run-id", "artifact-id", "artifact-type", "title", "content"];
  }

  get runId(): string {
    return this.getAttribute("run-id") ?? "";
  }
  get artifactId(): string {
    return this.getAttribute("artifact-id") ?? "";
  }
  get artifactType(): string {
    return this.getAttribute("artifact-type") ?? "display";
  }
  get artifactTitle(): string {
    return this.getAttribute("title") ?? "";
  }
  get contentStr(): string {
    return this.getAttribute("content") ?? "";
  }

  private submitted = false;

  connectedCallback(): void {
    this.render();
  }

  attributeChangedCallback(): void {
    if (this.isConnected) {
      this.render();
    }
  }

  private render(): void {
    const type = this.artifactType.toLowerCase().replace(/-/g, "_");

    switch (type) {
      case "form":
        this.renderForm();
        break;
      case "confirm":
        this.renderConfirm();
        break;
      case "select":
        this.renderSelect();
        break;
      case "text_input":
      case "text-input":
        this.renderTextInput();
        break;
      case "display":
      default:
        this.renderDisplay();
        break;
    }
  }

  // ── Shared submit logic ──────────────────────────────────────────────────

  private async submitResponse(data: ArtifactSubmitResult): Promise<void> {
    if (this.submitted) return;
    this.submitted = true;

    const url = `/api/uar/runs/${this.runId}/artifact-response`;
    try {
      const res = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          artifact_id: this.artifactId,
          response: data,
        }),
      });

      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
      }

      this.showSubmitSuccess();
      this.dispatchEvent(
        new CustomEvent("a2ui-response-submitted", {
          bubbles: true,
          composed: true,
          detail: { runId: this.runId, artifactId: this.artifactId, response: data },
        }),
      );
    } catch (err) {
      this.submitted = false;
      const errEl = this.querySelector(".a2ui-error");
      if (errEl) errEl.textContent = `Submission failed: ${err}`;
    }
  }

  private showSubmitSuccess(): void {
    this.innerHTML = `
      <div class="a2ui-artifact a2ui-submitted">
        <div class="a2ui-success-icon">✓</div>
        <p class="a2ui-success-msg">Response submitted. The agent is continuing…</p>
      </div>
    `;
  }

  // ── Form renderer ────────────────────────────────────────────────────────

  private renderForm(): void {
    let schema: ArtifactFormSchema = {};
    try {
      schema = JSON.parse(this.contentStr) as ArtifactFormSchema;
    } catch {
      schema = {};
    }

    const fields = schema.properties ?? {};
    const required = new Set(schema.required ?? []);

    const fieldsHtml = Object.entries(fields)
      .map(([key, field]) => {
        const label = field.title ?? key;
        const req = required.has(key) ? " *" : "";
        const desc = field.description
          ? `<p class="a2ui-field-desc">${field.description}</p>`
          : "";

        if (field.enum?.length) {
          const options = field.enum
            .map((v) => `<option value="${v}">${v}</option>`)
            .join("");
          return `
            <div class="a2ui-field">
              <label class="a2ui-label" for="a2ui-${key}">${label}${req}</label>
              ${desc}
              <select class="a2ui-select" id="a2ui-${key}" name="${key}" ${required.has(key) ? "required" : ""}>
                <option value="">Select…</option>
                ${options}
              </select>
            </div>`;
        }

        if (field.type === "boolean") {
          return `
            <div class="a2ui-field a2ui-field-checkbox">
              <input type="checkbox" class="a2ui-checkbox" id="a2ui-${key}" name="${key}" />
              <label class="a2ui-label" for="a2ui-${key}">${label}${req}</label>
              ${desc}
            </div>`;
        }

        const inputType =
          field.type === "integer" || field.type === "number" ? "number" : "text";
        return `
          <div class="a2ui-field">
            <label class="a2ui-label" for="a2ui-${key}">${label}${req}</label>
            ${desc}
            <input class="a2ui-input" type="${inputType}" id="a2ui-${key}" name="${key}"
              ${required.has(key) ? "required" : ""}
              ${field.minimum != null ? `min="${field.minimum}"` : ""}
              ${field.maximum != null ? `max="${field.maximum}"` : ""}
            />
          </div>`;
      })
      .join("");

    this.innerHTML = `
      <div class="a2ui-artifact a2ui-form">
        <div class="a2ui-header">
          <span class="a2ui-badge">Agent Input Required</span>
          <h3 class="a2ui-title">${this.artifactTitle}</h3>
          ${schema.description ? `<p class="a2ui-desc">${schema.description}</p>` : ""}
        </div>
        <form class="a2ui-form-body">
          ${fieldsHtml}
          <p class="a2ui-error" style="display:none;color:var(--color-error,#ef4444)"></p>
          <div class="a2ui-actions">
            <button type="submit" class="a2ui-btn a2ui-btn-primary">Submit</button>
          </div>
        </form>
      </div>
    `;

    const form = this.querySelector("form");
    form?.addEventListener("submit", (e) => {
      e.preventDefault();
      const fd = new FormData(form);
      const data: Record<string, unknown> = {};
      for (const [key, field] of Object.entries(fields)) {
        if (field.type === "boolean") {
          data[key] = (this.querySelector(`#a2ui-${key}`) as HTMLInputElement)?.checked ?? false;
        } else if (field.type === "integer") {
          data[key] = parseInt(fd.get(key) as string, 10);
        } else if (field.type === "number") {
          data[key] = parseFloat(fd.get(key) as string);
        } else {
          data[key] = fd.get(key);
        }
      }
      void this.submitResponse(data);
    });
  }

  // ── Confirm renderer ─────────────────────────────────────────────────────

  private renderConfirm(): void {
    let parsed: { message?: string; accept_label?: string; cancel_label?: string } = {};
    try {
      parsed = JSON.parse(this.contentStr) as typeof parsed;
    } catch {
      parsed = {};
    }

    const message = parsed.message ?? this.artifactTitle ?? "Please confirm to continue.";
    const acceptLabel = parsed.accept_label ?? "Accept";
    const cancelLabel = parsed.cancel_label ?? "Cancel";

    this.innerHTML = `
      <div class="a2ui-artifact a2ui-confirm">
        <div class="a2ui-header">
          <span class="a2ui-badge">Confirmation Required</span>
          <h3 class="a2ui-title">${this.artifactTitle}</h3>
        </div>
        <p class="a2ui-confirm-msg">${message}</p>
        <p class="a2ui-error" style="display:none;color:var(--color-error,#ef4444)"></p>
        <div class="a2ui-actions">
          <button class="a2ui-btn a2ui-btn-secondary" data-action="cancel">${cancelLabel}</button>
          <button class="a2ui-btn a2ui-btn-primary" data-action="accept">${acceptLabel}</button>
        </div>
      </div>
    `;

    this.querySelector('[data-action="accept"]')?.addEventListener("click", () => {
      void this.submitResponse({ accepted: true });
    });
    this.querySelector('[data-action="cancel"]')?.addEventListener("click", () => {
      void this.submitResponse({ accepted: false });
    });
  }

  // ── Select renderer ──────────────────────────────────────────────────────

  private renderSelect(): void {
    let parsed: {
      prompt?: string;
      options?: Array<{ value: string; label: string; description?: string }>;
    } = {};
    try {
      parsed = JSON.parse(this.contentStr) as typeof parsed;
    } catch {
      parsed = {};
    }

    const prompt = parsed.prompt ?? "Choose one option:";
    const options = parsed.options ?? [];

    const optionsHtml = options
      .map(
        (opt) => `
        <label class="a2ui-option-label">
          <input type="radio" name="a2ui-select-value" value="${opt.value}" class="a2ui-radio" />
          <span class="a2ui-option-text">
            <strong>${opt.label}</strong>
            ${opt.description ? `<small>${opt.description}</small>` : ""}
          </span>
        </label>`,
      )
      .join("");

    this.innerHTML = `
      <div class="a2ui-artifact a2ui-select">
        <div class="a2ui-header">
          <span class="a2ui-badge">Selection Required</span>
          <h3 class="a2ui-title">${this.artifactTitle}</h3>
          <p class="a2ui-desc">${prompt}</p>
        </div>
        <div class="a2ui-options">${optionsHtml}</div>
        <p class="a2ui-error" style="display:none;color:var(--color-error,#ef4444)"></p>
        <div class="a2ui-actions">
          <button class="a2ui-btn a2ui-btn-primary" id="a2ui-select-submit">Confirm Selection</button>
        </div>
      </div>
    `;

    this.querySelector("#a2ui-select-submit")?.addEventListener("click", () => {
      const selected = this.querySelector<HTMLInputElement>(
        'input[name="a2ui-select-value"]:checked',
      );
      if (!selected) {
        const errEl = this.querySelector(".a2ui-error") as HTMLElement;
        errEl.style.display = "block";
        errEl.textContent = "Please select an option before confirming.";
        return;
      }
      void this.submitResponse({ value: selected.value });
    });
  }

  // ── Text input renderer ──────────────────────────────────────────────────

  private renderTextInput(): void {
    let parsed: { prompt?: string; placeholder?: string; multiline?: boolean } = {};
    try {
      parsed = JSON.parse(this.contentStr) as typeof parsed;
    } catch {
      parsed = {};
    }

    const prompt = parsed.prompt ?? this.artifactTitle ?? "Enter your response:";
    const placeholder = parsed.placeholder ?? "";
    const multiline = parsed.multiline ?? false;

    const inputHtml = multiline
      ? `<textarea class="a2ui-textarea" placeholder="${placeholder}" rows="4"></textarea>`
      : `<input type="text" class="a2ui-input" placeholder="${placeholder}" />`;

    this.innerHTML = `
      <div class="a2ui-artifact a2ui-text-input">
        <div class="a2ui-header">
          <span class="a2ui-badge">Input Required</span>
          <h3 class="a2ui-title">${this.artifactTitle}</h3>
          <p class="a2ui-desc">${prompt}</p>
        </div>
        ${inputHtml}
        <p class="a2ui-error" style="display:none;color:var(--color-error,#ef4444)"></p>
        <div class="a2ui-actions">
          <button class="a2ui-btn a2ui-btn-primary" id="a2ui-text-submit">Send</button>
        </div>
      </div>
    `;

    this.querySelector("#a2ui-text-submit")?.addEventListener("click", () => {
      const input = this.querySelector<HTMLInputElement | HTMLTextAreaElement>(
        ".a2ui-input, .a2ui-textarea",
      );
      const text = input?.value.trim() ?? "";
      if (!text) {
        const errEl = this.querySelector(".a2ui-error") as HTMLElement;
        errEl.style.display = "block";
        errEl.textContent = "Please enter a response before submitting.";
        return;
      }
      void this.submitResponse({ text });
    });
  }

  // ── Display renderer ─────────────────────────────────────────────────────

  private renderDisplay(): void {
    let content = this.contentStr;
    // Try to parse as JSON with a `content` field first.
    try {
      const parsed = JSON.parse(content) as { content?: string; format?: string };
      if (typeof parsed.content === "string") {
        content = parsed.content;
      }
    } catch {
      // Use raw content string directly
    }

    // Basic markdown-to-HTML (inline code, bold, italic, headings, lists)
    const html = content
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
      .replace(/\*([^*]+)\*/g, "<em>$1</em>")
      .replace(/^### (.+)$/gm, "<h3>$1</h3>")
      .replace(/^## (.+)$/gm, "<h2>$1</h2>")
      .replace(/^# (.+)$/gm, "<h1>$1</h1>")
      .replace(/\n/g, "<br />");

    this.innerHTML = `
      <div class="a2ui-artifact a2ui-display">
        ${this.artifactTitle ? `<h3 class="a2ui-title">${this.artifactTitle}</h3>` : ""}
        <div class="a2ui-display-content">${html}</div>
      </div>
    `;
  }
}

// Register the custom element
customElements.define("a2ui-artifact", A2uiArtifact);
