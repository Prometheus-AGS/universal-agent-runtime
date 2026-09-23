# Goals — skills-a2ui-library-and-runtime-observability › agui-a2ui-selection-architecture › classify-readiness-diagnostic-websocket-frame

- Capture only sanitized FIN/opcode/mask/length metadata for the rejected WebSocket frame without retaining payloads or credentials.
- Classify the observed frame against the WebSocket protocol and the existing collector behavior without broadening protocol handling.
- Produce independently reviewed diagnostic evidence and a minimum next-action recommendation; do not rerun readiness or change product code, configuration, dependencies, databases, or services without separate authority.
