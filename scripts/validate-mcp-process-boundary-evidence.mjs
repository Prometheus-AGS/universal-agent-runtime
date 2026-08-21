#!/usr/bin/env node

import { readFileSync } from "node:fs";

const [crashStreamPath, timeoutStreamPath, processTracePath] = process.argv.slice(2);
if (!crashStreamPath || !timeoutStreamPath || !processTracePath) {
  console.error(
    "usage: validate-mcp-process-boundary-evidence.mjs <crash-sse> <timeout-sse> <process-trace-jsonl>",
  );
  process.exit(2);
}

const failures = [];

function readStreamedToolResults(path, label) {
  const results = [];
  for (const line of readFileSync(path, "utf8").split("\n")) {
    if (!line.startsWith("data: ") || line === "data: [DONE]") continue;
    let chunk;
    try {
      chunk = JSON.parse(line.slice("data: ".length));
    } catch (error) {
      failures.push(`${label} contains invalid SSE JSON: ${error.message}`);
      continue;
    }
    for (const choice of chunk.choices ?? []) {
      for (const result of choice.delta?.tool_results ?? []) results.push(result);
    }
  }
  return results;
}

function requireSingleFailure(path, label) {
  const results = readStreamedToolResults(path, label).filter(
    (result) => result.name === "resilience__echo",
  );
  if (
    results.length !== 1
    || results[0].success !== false
    || !String(results[0].content).includes("tools/call failed")
  ) {
    failures.push(`${label} must contain exactly one explicit failed resilience__echo result`);
  }
  if (results.some((result) => result.success === true)) {
    failures.push(`${label} reports the failed resilience__echo call as successful`);
  }
}

requireSingleFailure(crashStreamPath, "crash stream");
requireSingleFailure(timeoutStreamPath, "timeout stream");

const traceRows = readFileSync(processTracePath, "utf8")
  .split("\n")
  .filter(Boolean)
  .map((line, index) => {
    try {
      return JSON.parse(line);
    } catch (error) {
      failures.push(`process trace line ${index + 1} is invalid JSON: ${error.message}`);
      return null;
    }
  })
  .filter(Boolean);

const expectedModes = ["echo", "crash", "echo", "timeout", "echo"];
const observedModes = traceRows.map((row) => row.mode);
if (JSON.stringify(observedModes) !== JSON.stringify(expectedModes)) {
  failures.push(
    `process trace must execute ${expectedModes.join(",")} exactly once; observed ${observedModes.join(",")}`,
  );
}

if (traceRows.length === expectedModes.length) {
  const pids = traceRows.map((row) => row.pid);
  if (!pids.every((pid) => Number.isInteger(pid) && pid > 0)) {
    failures.push("process trace contains an invalid MCP process id");
  } else {
    if (pids[0] !== pids[1]) failures.push("crash was not executed by the initially connected MCP process");
    if (pids[1] === pids[2]) failures.push("post-crash call did not use a replacement MCP process");
    if (pids[2] !== pids[3]) failures.push("timeout was not executed by the post-crash MCP process");
    if (pids[3] === pids[4]) failures.push("post-timeout call did not use a replacement MCP process");
  }
}

if (failures.length) {
  console.error(`MCP process-boundary evidence validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}

console.log(
  "MCP_PROCESS_BOUNDARY_EVIDENCE_PASS crash_failure_events=1 timeout_failure_events=1 crash_calls=1 timeout_calls=1 reconnects=2",
);
