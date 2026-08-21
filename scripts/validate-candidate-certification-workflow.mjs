#!/usr/bin/env node

import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

// Kept under its historical filename so existing package scripts remain
// compatible. The contract is now deliberately local; no workflow is read.
const certifier = readFileSync(new URL("./certify-release-candidate.sh", import.meta.url), "utf8");
const packager = readFileSync(new URL("./package-candidate-certification-local.sh", import.meta.url), "utf8");
const bundleValidator = readFileSync(new URL("./validate-candidate-certification-bundle.mjs", import.meta.url), "utf8");
const mcpEvidenceValidator = fileURLToPath(
  new URL("./validate-mcp-process-boundary-evidence.mjs", import.meta.url),
);
const failures = [];

for (const contract of [
  "release-manifest.json",
  "candidate manifest/source SHA",
  "UAR_SOAK_DURATION_SECONDS",
  "-w '%{http_code} %{time_total}\\n'",
  "chat_request mcp-crash true",
  "chat_request mcp-timeout true",
  "mcp-process-trace.jsonl",
  "validate-mcp-process-boundary-evidence.mjs",
]) {
  if (!certifier.includes(contract)) failures.push(`missing installed candidate certifier contract: ${contract}`);
}

const fixtureDirectory = mkdtempSync(join(tmpdir(), "uar-mcp-boundary-contract-"));
try {
  const crashStream = join(fixtureDirectory, "mcp-crash.sse");
  const timeoutStream = join(fixtureDirectory, "mcp-timeout.sse");
  const trace = join(fixtureDirectory, "mcp-process-trace.jsonl");
  const toolResultStream = (...successValues) =>
    successValues
      .map(
        (success) =>
          `data: ${JSON.stringify({
            choices: [
              {
                delta: {
                  tool_results: [
                    {
                      name: "resilience__echo",
                      content: "Error: tools/call failed for resilience::echo",
                      success,
                    },
                  ],
                },
              },
            ],
          })}`,
      )
      .join("\n\n") + "\n\ndata: [DONE]\n\n";
  const positiveTrace = [
    { pid: 100, request_id: 1, mode: "echo" },
    { pid: 100, request_id: 2, mode: "crash" },
    { pid: 200, request_id: 1, mode: "echo" },
    { pid: 200, request_id: 2, mode: "timeout" },
    { pid: 300, request_id: 1, mode: "echo" },
  ];
  const writeTrace = (rows) =>
    writeFileSync(trace, rows.map(JSON.stringify).join("\n") + "\n");
  const expectRejected = (label) => {
    if (replay().status === 0) failures.push(`${label} negative control was accepted`);
  };

  writeFileSync(crashStream, toolResultStream(false));
  writeFileSync(timeoutStream, toolResultStream(false));
  writeTrace(positiveTrace);

  const replay = () =>
    spawnSync(process.execPath, [mcpEvidenceValidator, crashStream, timeoutStream, trace], {
      encoding: "utf8",
    });
  const positive = replay();
  if (positive.status !== 0 || !positive.stdout.includes("MCP_PROCESS_BOUNDARY_EVIDENCE_PASS")) {
    failures.push(`MCP process-boundary positive fixture failed: ${positive.stderr.trim()}`);
  }

  writeFileSync(crashStream, toolResultStream(true));
  expectRejected("MCP successful-crash");
  writeFileSync(crashStream, toolResultStream(false, false));
  expectRejected("MCP duplicate-failure-event");
  writeFileSync(crashStream, toolResultStream(false));

  writeTrace([...positiveTrace, { pid: 300, request_id: 2, mode: "crash" }]);
  expectRejected("MCP duplicate-process-execution");
  writeTrace(positiveTrace.slice(0, -1));
  expectRejected("MCP missing-post-timeout-transition");
  writeTrace(positiveTrace.map((row, index) => (index === 2 ? { ...row, pid: 100 } : row)));
  expectRejected("MCP stale-post-crash-process");
  writeTrace(positiveTrace.map((row, index) => (index === 4 ? { ...row, pid: 200 } : row)));
  expectRejected("MCP stale-post-timeout-process");
  console.log(
    "MCP_PROCESS_BOUNDARY_CONTRACT_PASS positive=1 negative_controls=6 success_substitution=reject duplicate_event=reject duplicate_execution=reject missing_transition=reject stale_after_crash=reject stale_after_timeout=reject",
  );
} finally {
  rmSync(fixtureDirectory, { recursive: true, force: true });
}
for (const contract of [
  "checkout must be clean",
  "validate-release-manifest.mjs",
  "validate-candidate-certification.mjs",
  "candidate-certification-manifest.json",
  "CANDIDATE_CERTIFICATION_SHA256SUMS",
  "cosign sign-blob --yes",
  "No candidate tag, release, archive, or image was built, uploaded, or promoted",
]) {
  if (!packager.includes(contract)) failures.push(`missing local candidate packager contract: ${contract}`);
}
for (const contract of [
  "candidate certification builder must be local",
  "candidate certification builder receipt digest mismatch",
  "missing candidate certification checksum signature bundle",
  "candidate certification asset set is not exact",
]) {
  if (!bundleValidator.includes(contract)) failures.push(`missing candidate bundle validator contract: ${contract}`);
}
for (const prohibited of ["actions/runs/", ".github/workflows/", "signer-workflow", "github.run_id", "github.workflow_ref"]) {
  if (certifier.includes(prohibited) || packager.includes(prohibited) || bundleValidator.includes(prohibited)) {
    failures.push(`candidate certification must not depend on GitHub Actions: ${prohibited}`);
  }
}

if (failures.length) {
  console.error(`Local candidate certification validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log("Local installed-candidate certification, packaging, and bundle contracts passed.");
