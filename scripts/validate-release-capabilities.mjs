import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const directTree = execFileSync(
  "cargo",
  [
    "tree",
    "--package",
    "universal-agent-runtime",
    "--no-default-features",
    "--features",
    "in-memory-backend",
    "--edges",
    "normal",
    "--depth",
    "1",
    "--prefix",
    "none",
  ],
  { cwd: root, encoding: "utf8" },
);

const forbiddenDirectDependencies = [
  "cedar-policy",
  "fastembed",
  "kreuzberg",
  "metrics-exporter-prometheus",
  "opentelemetry-otlp",
  "sycophancy-core",
  "tonic",
  "tracing-opentelemetry",
  "utoipa-swagger-ui",
  "wasmtime",
];

for (const dependency of forbiddenDirectDependencies) {
  const pattern = new RegExp(`^${dependency.replaceAll("-", "\\-")} v`, "m");
  if (pattern.test(directTree)) {
    throw new Error(`disabled capability retained direct dependency: ${dependency}`);
  }
}

const server = readFileSync(resolve(root, "src/server.rs"), "utf8");
for (const guardedSurface of ["a2a_routes", "grpc_handle"]) {
  const pattern = new RegExp(
    `#\\[cfg\\(feature = "a2a-transport"\\)\\][\\s\\S]{0,80}(?:let )?${guardedSurface}`,
  );
  if (!pattern.test(server)) {
    throw new Error(`A2A public surface lacks capability guard: ${guardedSurface}`);
  }
}

console.log(
  `Release capability boundaries valid (${forbiddenDirectDependencies.length} disabled direct dependencies).`,
);
