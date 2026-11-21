export const STUB_PORT = 4600;
// Per-provider stub instances for the S1 live-model-choice features: every
// synthetic provider has its own stub so a request log names the provider that
// served it. `STUB_PORT` stays the stub behind the `openai` provider.
export const STUB_PORT_B = 4601;
export const STUB_PORT_C = 4602;
export const APP_PORT = 3002; // distinct from tests/e2e's 3001 so both suites can run concurrently
export const STUB_BASE_URL = `http://127.0.0.1:${STUB_PORT}`;
export const STUB_BASE_URL_B = `http://127.0.0.1:${STUB_PORT_B}`;
export const STUB_BASE_URL_C = `http://127.0.0.1:${STUB_PORT_C}`;
export const APP_BASE_URL = `http://127.0.0.1:${APP_PORT}`;
