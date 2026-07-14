import { UarClient } from "../src/index.js";
const client = new UarClient(process.env.UAR_URL ?? "http://localhost:1906");
const run = await client.runs.create({ artifact: { name: "assistant", version: "1" }, input: "Draft a release note" });
for await (const event of client.runs.stream(run.run_id)) console.log(event);
