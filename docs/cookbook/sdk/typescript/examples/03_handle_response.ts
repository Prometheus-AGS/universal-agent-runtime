import { UarClient } from "../../../../../sdks/typescript/src/index.js";

const client = new UarClient(process.env.UAR_BASE_URL ?? "http://localhost:1906");
const response = await client.chat.complete({
  messages: [{ role: "user", content: "Hello from the TypeScript cookbook" }],
});

console.log("Response ID:", response.id);
console.log("Choices:", response.choices.length);
if (response.choices[0]?.message?.content) {
  console.log("Content:", response.choices[0].message.content);
} else {
  console.log("No content in response");
}
