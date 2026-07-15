import type { ReactNode } from "react";
import { UarSurface } from "@prometheus-ags/a2ui-uar";
import { useInspector } from "../hooks/use-inspector";
import type { InspectorStore } from "../stores/inspector-store";
import "../styles.css";

export interface InspectorPanelProps { store: InspectorStore; renderPreview?: (surface: NonNullable<ReturnType<typeof useInspector>["surface"]>) => ReactNode; }

export function InspectorPanel({ store, renderPreview = (surface) => <UarSurface surface={surface} /> }: InspectorPanelProps) {
  const state = useInspector(store);
  const selected = state.messages.find(({ id }) => id === state.selectedId);
  const filter = state.filter.trim().toLowerCase();
  const messages = state.messages.filter((message) => !filter || message.kind.toLowerCase().includes(filter) || JSON.stringify(message.raw).toLowerCase().includes(filter));
  return <main className="a2ui-inspector" aria-label="A2UI Inspector">
    <header className="a2ui-inspector__toolbar">
      <div><strong>A2UI Inspector</strong><span className={`a2ui-inspector__status is-${state.connection}`} role="status">{state.connection}</span></div>
      <label>Filter messages<input value={state.filter} onChange={(event) => state.setFilter(event.target.value)} placeholder="Type or payload" /></label>
      <button type="button" aria-pressed={state.frozen} onClick={state.toggleFreeze}>{state.frozen ? "Resume preview" : "Freeze preview"}</button>
      <button type="button" onClick={state.clear} disabled={!state.messages.length}>Clear</button>
      {state.connection === "error" || state.connection === "disconnected" ? <button type="button" onClick={state.retry}>Retry connection</button> : null}
    </header>
    {state.frozen ? <div className="a2ui-inspector__freeze" role="status"><strong>Preview frozen</strong><span>{state.queued} queued message{state.queued === 1 ? "" : "s"}. Stream ingestion continues.</span></div> : null}
    <div className="a2ui-inspector__workspace">
      <nav className="a2ui-inspector__timeline" aria-label="Captured messages">
        <div className="a2ui-inspector__timeline-meta"><span>{messages.length} shown</span><span>{state.dropped} dropped</span></div>
        {messages.length ? <ol>{messages.map((message) => <li key={message.id}><button type="button" className={message.id === state.selectedId ? "is-selected" : ""} aria-current={message.id === state.selectedId ? "true" : undefined} onClick={() => state.select(message.id)}><span>{message.kind}</span><time dateTime={message.receivedAt}>{new Date(message.receivedAt).toLocaleTimeString()}</time>{message.valid ? null : <span className="a2ui-inspector__invalid">Invalid</span>}</button></li>)}</ol> : <p className="a2ui-inspector__empty">No matching messages. Connect a development stream or adjust the filter.</p>}
      </nav>
      <section className="a2ui-inspector__preview" aria-labelledby="preview-heading">
        <h2 id="preview-heading">Preview</h2>
        {selected && !selected.valid ? <div role="alert" className="a2ui-inspector__error"><strong>Message could not be applied</strong><p>{selected.error}</p><span>The last valid preview is preserved.</span></div> : null}
        {state.surface ? <div className="a2ui-inspector__preview-canvas">{renderPreview(state.surface)}</div> : <div className="a2ui-inspector__empty"><strong>No surface yet</strong><p>Waiting for a valid createSurface message.</p></div>}
      </section>
      <section className="a2ui-inspector__source" aria-labelledby="source-heading">
        <div><h2 id="source-heading">Source JSON</h2><button type="button" onClick={() => void state.copySelected()} disabled={!selected}>Copy JSON</button></div>
        <pre tabIndex={0}>{selected ? JSON.stringify(selected.raw, null, 2) : "Select a captured message."}</pre>
      </section>
    </div>
  </main>;
}
