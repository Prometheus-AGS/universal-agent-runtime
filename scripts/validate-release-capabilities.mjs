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
const dockerfile = readFileSync(resolve(root, "Dockerfile"), "utf8");
const dockerignore = readFileSync(resolve(root, ".dockerignore"), "utf8");
const localCertifier = readFileSync(
  resolve(root, "scripts/certify-operational-resilience-local.sh"),
  "utf8",
);
if (!dockerfile.includes('--features "server-full,postgres-backend"')) {
  throw new Error("Dockerfile must build the authoritative server-full bundle");
}
if (dockerfile.includes("memory-palace")) {
  throw new Error("Dockerfile references the removed memory-palace feature");
}
if (!/^ARG TARGETARCH$/m.test(dockerfile) || /^ARG TARGETARCH=/m.test(dockerfile)) {
  throw new Error("Dockerfile must inherit BuildKit TARGETARCH without a hard-coded default");
}
for (const ignoredArtifact of ["**/node_modules", "**/dist", "**/.turbo"]) {
  if (!dockerignore.split("\n").includes(ignoredArtifact)) {
    throw new Error(`Docker build context must recursively exclude ${ignoredArtifact}`);
  }
}
if (!dockerfile.includes("cd packages/prometheus-entity-management")) {
  throw new Error("Dockerfile must install the nested entity-management workspace");
}
for (const requiredPackage of [
  "@prometheus-ags/entity-graph-core",
  "@prometheus-ags/prometheus-entity-management",
]) {
  if (!dockerfile.includes(`pnpm --filter ${requiredPackage} build`)) {
    throw new Error(`Dockerfile must build shipped workspace package: ${requiredPackage}`);
  }
}
if (/pnpm\s+-r\s+--filter\s+["']\.\/packages\/\*["']\s+build/.test(dockerfile)) {
  throw new Error("Dockerfile must not build every nested frontend workspace package");
}
if (!localCertifier.includes("--bin universal-agent-runtime --features server-full")) {
  throw new Error("native release archives must build the authoritative server-full bundle");
}
if (!localCertifier.includes('cp -R static "$package_dir/static"')) {
  throw new Error("native Unix release archives must contain the React bundle");
}
if (!localCertifier.includes("crates/prometheus-skill-system/skills")) {
  throw new Error("native release archives must contain the built-in skill pack");
}
if (!localCertifier.includes("src/uar/runtime/matching/models")) {
  throw new Error("native release archives must contain local model assets");
}
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
