import { UarClient } from "../src/index.js";
const client = new UarClient(process.env.UAR_URL ?? "http://localhost:1906");
console.log(await client.tools.execute("web::search", { query: "UAR" }));
