// Local release gate only. Load the immutable published @ag-ui/core 1.0.2 schema.
import { pathToFileURL } from 'node:url';
import { createInterface } from 'node:readline';
const modulePath = process.env.UAR_AGUI_SCHEMA_MODULE;
if (!modulePath) throw new Error('UAR_AGUI_SCHEMA_MODULE must identify the pinned release schema');
const { EventSchema, RunAgentInputSchema } = await import(pathToFileURL(modulePath).href);
if (!EventSchema || !RunAgentInputSchema) throw new Error('Pinned package does not export required schemas');
for await (const line of createInterface({ input: process.stdin, crlfDelay: Infinity })) {
  if (!line.trim()) continue;
  const value = JSON.parse(line);
  (value.type === undefined ? RunAgentInputSchema : EventSchema).parse(value);
  process.stdout.write('{"valid":true}\n');
}
