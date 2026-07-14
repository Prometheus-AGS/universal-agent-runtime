import { z } from "zod";
import { UarClient } from "../src/index.js";
const client = new UarClient(process.env.UAR_URL ?? "http://localhost:1906");
console.log(await client.chat.structured({ messages: [{ role: "user", content: "Return a task" }] }, z.object({ title: z.string(), done: z.boolean() })));
