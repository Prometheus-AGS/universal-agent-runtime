import { UarClient } from "../../src/index.js";
const client = new UarClient(process.env.UAR_URL ?? "http://localhost:1906", { apiKey: process.env.UAR_API_KEY });
export async function POST(request: Request): Promise<Response> {
  const { message } = await request.json() as { message: string };
  return Response.json(await client.chat.complete({ messages: [{ role: "user", content: message }] }));
}
