#!/usr/bin/env python3
"""Run one A2A release tuple from the runtime container's network namespace.

The installed image remains the client image. This entrypoint is needed only
for the loopback-only governance-disabled profile: host-published Docker ports
cannot reach a process bound to 127.0.0.1 inside a different namespace.
"""

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/release"))
import uar_1_1_acceptance as acceptance


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--governance", choices=("enforced", "disabled"), required=True)
    parser.add_argument("--transport", choices=("jsonrpc", "grpc"), required=True)
    parser.add_argument("--decision", choices=("approve", "deny"), required=True)
    parser.add_argument("--peer-port", type=int, required=True)
    parser.add_argument("--peer-url", required=True)
    parser.add_argument("--peer-trace", type=Path, required=True)
    parser.add_argument("--provider-trace", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--result", type=Path, required=True)
    args = parser.parse_args()

    trusted_token = os.environ["UAR_RELEASE_TRUSTED_TOKEN"]
    limited_token = os.environ["UAR_RELEASE_LIMITED_TOKEN"]
    host_token = os.environ["UAR_RELEASE_HOST_TOKEN"]
    class LoopbackRuntime:
        base_url = "http://127.0.0.1:1906"
        grpc_target = "127.0.0.1:50051"

        def gateway_url(self, port):
            if port != args.peer_port:
                raise RuntimeError("loopback client received an unexpected peer port")
            return args.peer_url

    runtime = LoopbackRuntime()
    owner = acceptance.Http(runtime.base_url, trusted_token, "release-settings-admin")
    acceptance.set_governance(owner, args.governance == "enforced")
    protected_agent, dynamic_agent = acceptance.configure_a2a_runtime(
        runtime, args.peer_port, trusted_token, host_token)
    environment = dict(
        os.environ,
        UAR_A2A_TRANSPORT=args.transport,
        UAR_A2A_GOVERNANCE=args.governance,
        UAR_A2A_BASE_URL=runtime.base_url,
        UAR_A2A_GRPC_TARGET=runtime.grpc_target,
        UAR_A2A_TOKEN=trusted_token,
        UAR_A2A_ROLE_LIMITED_TOKEN=limited_token,
        UAR_A2A_TENANT_ID=acceptance.TENANT,
        UAR_A2A_AGENT_ID=protected_agent,
        UAR_A2A_MCP_TRACE_PATH=str(args.peer_trace),
        UAR_A2A_PROVIDER_TRACE_PATH=str(args.provider_trace),
        UAR_A2A_PROTECTED_PROMPT_MARKER="UAR_PROTECTED_PROMPT_BODY_379",
        UAR_A2A_APPROVAL_INPUT=json.dumps({
            "dot_action": "tool", "tool": "a2a-qualification__effect", "arguments": {},
        }),
        UAR_A2A_DYNAMIC_AGENT_ID=dynamic_agent,
        UAR_A2A_DYNAMIC_SKILL_ID=acceptance.PROTECTED_SKILL,
        UAR_A2A_GRPCURL="/tools/grpcurl",
        UAR_A2A_GRPCURL_ARGS="-plaintext",
    )
    fixture = ROOT / "tests/fixtures/a2a_v1_approval_governance.py"
    completed = subprocess.run(
        [sys.executable, str(fixture), "--transport", args.transport,
         "--governance", args.governance, "--decision", args.decision,
         "--receipt-path", str(args.receipt)],
        cwd=ROOT,
        env=environment,
        check=False,
        text=True,
        capture_output=True,
    )
    if completed.returncode:
        raise RuntimeError(
            "A2A fixture failed: " + json.dumps(
                {"stdout": completed.stdout, "stderr": completed.stderr}, indent=2
            )
        )
    result = {
        "protected_agent": protected_agent,
        "dynamic_agent": dynamic_agent,
        "receipt": json.loads(args.receipt.read_text()),
    }
    args.result.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
