import { LitElement, html } from "lit";
import { property } from "lit/decorators.js";
import type { ComponentApi, SurfaceModel } from "@prometheus-ags/a2ui-core/v0_9";
import { renderSemanticSurface, type SemanticRendererHandle } from "@prometheus-ags/a2ui-core/semantic-dom";

export class A2uiLitSurface extends LitElement {
  @property({ attribute: false }) surface?: SurfaceModel<ComponentApi>;
  private handle?: SemanticRendererHandle;

  protected createRenderRoot(): HTMLElement | DocumentFragment { return this; }
  protected render() { return html`<div data-a2ui-lit-surface aria-label="A2UI surface"></div>`; }
  protected updated(): void {
    this.handle?.dispose();
    const target = this.querySelector<HTMLElement>("[data-a2ui-lit-surface]");
    if (target && this.surface) this.handle = renderSemanticSurface(this.surface, target);
  }
  disconnectedCallback(): void { this.handle?.dispose(); super.disconnectedCallback(); }
}

if (!customElements.get("a2ui-lit-surface")) customElements.define("a2ui-lit-surface", A2uiLitSurface);

declare global { interface HTMLElementTagNameMap { "a2ui-lit-surface": A2uiLitSurface; } }
