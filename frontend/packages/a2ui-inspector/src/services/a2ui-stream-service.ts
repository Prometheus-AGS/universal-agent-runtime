import type { InspectorConnection } from "../types";

export interface A2uiStreamService {
  connect(onMessage: (payload: unknown) => void, onStatus: (status: InspectorConnection) => void): () => void;
}

export function createEventSourceService(url: string, redact: (payload: unknown) => unknown = (value) => value): A2uiStreamService {
  return {
    connect(onMessage, onStatus) {
      onStatus("connecting");
      const source = new EventSource(url);
      source.onopen = () => onStatus("connected");
      source.onmessage = (event) => {
        try { onMessage(redact(JSON.parse(event.data))); }
        catch { onMessage({ inspectorError: "Message was not valid JSON", source: event.data }); }
      };
      source.onerror = () => onStatus(source.readyState === EventSource.CLOSED ? "disconnected" : "error");
      return () => { source.close(); onStatus("disconnected"); };
    },
  };
}
