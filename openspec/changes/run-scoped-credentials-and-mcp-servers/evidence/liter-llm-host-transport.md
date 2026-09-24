# Host-supplied provider transport inspection

Date: 2026-09-23

Source inspection found that liter-llm constructed authenticated clients through
`configure_outbound_client_builder`. That policy rejected cross-origin redirects,
but allowed policy-checked same-origin redirects. It disabled ambient proxies only
when the process-global outbound policy was active.

The request-scoped UAR path requires a stronger per-client boundary because its
endpoint and API key belong to one host request. Liter-llm commit
`e627af981bcb06c7fc5da027731c182b044e25d1` adds opt-in transport flags for no
proxy and no redirects and supports redacting a host-owned base URL from
`ClientConfig` debug output. It also removes full provider URLs from HTTP trace
spans and avoids logging untrusted transport or response values while retaining
method, status, and retry telemetry. UAR enables all three only when
`LlmConfig::host_supplied_connection` is true. Existing clients retain their
previous defaults.

No redirect integration scenario was run at this implementation boundary. It
remains part of the phase integration gate in task 6.1.
