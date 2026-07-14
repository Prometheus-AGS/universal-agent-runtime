import { UarClient } from "../src/index.js";
const client = new UarClient(process.env.UAR_URL ?? "http://localhost:1906");
for await (const event of client.chat.stream({ messages: [{ role: "user", content: "Stream a haiku" }] })) console.log(event);
