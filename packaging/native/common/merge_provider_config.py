#!/usr/bin/env python3
"""Add native listener defaults and absent provider seeds without rewriting YAML."""

from __future__ import annotations

import argparse
import json
import os
import re
import stat
import tempfile
import urllib.error
import urllib.request
from pathlib import Path


PROVIDER_VARIABLES = {
    "KIMI_API_KEY",
    "MINIMAX_API_KEY",
    "DASHSCOPE_API_KEY",
    "MOONSHOT_API_KEY",
    "ZAI_API_KEY",
}
SAFE_MODEL_ID = re.compile(r"^[A-Za-z0-9._:/+\-]+$")
QWEN_MODEL_ID = "qwen3.8-max"
QWEN_MODEL_NAME = "Qwen3.8-Max"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", required=True, type=Path)
    parser.add_argument("--env-file", required=True, type=Path)
    parser.add_argument("--proxy-url", required=True)
    return parser.parse_args()


def present_provider_variables(path: Path) -> set[str]:
    present: set[str] = set()
    for line in path.read_text(encoding="utf-8-sig").splitlines():
        match = re.match(r"^([A-Z][A-Z0-9_]*)=(.*)$", line)
        if match and match.group(1) in PROVIDER_VARIABLES and match.group(2).strip():
            present.add(match.group(1))
    return present


def discover_proxy_models(proxy_url: str) -> list[str]:
    url = f"{proxy_url.rstrip('/')}/models"
    request = urllib.request.Request(url, headers={"Accept": "application/json"})
    try:
        with urllib.request.urlopen(request, timeout=5) as response:
            payload = json.load(response)
    except (OSError, ValueError, urllib.error.URLError) as error:
        print(f"local proxy inventory unavailable; proxy seed omitted: {type(error).__name__}", file=os.sys.stderr)
        return []

    raw_models = payload.get("data") if isinstance(payload, dict) else None
    if not isinstance(raw_models, list):
        print("local proxy inventory invalid; proxy seed omitted", file=os.sys.stderr)
        return []

    models = sorted(
        {
            item["id"]
            for item in raw_models
            if isinstance(item, dict)
            and isinstance(item.get("id"), str)
            and SAFE_MODEL_ID.fullmatch(item["id"])
        }
    )
    if not models:
        print("local proxy inventory empty; proxy seed omitted", file=os.sys.stderr)
    return models


def model_lines(model_id: str, display_name: str, context: int | None, output: int | None,
                vision: bool, tools: bool, reasoning: bool, structured: bool) -> list[str]:
    lines = [
        f"      - id: {json.dumps(model_id)}",
        f"        display_name: {json.dumps(display_name)}",
    ]
    if context is not None:
        lines.append(f"        context_window: {context}")
    lines.extend(
        [
            f"        supports_vision: {str(vision).lower()}",
            f"        supports_tools: {str(tools).lower()}",
            f"        supports_reasoning: {str(reasoning).lower()}",
            f"        supports_structured_output: {str(structured).lower()}",
            "        supports_streaming: true",
        ]
    )
    if output is not None:
        lines.append(f"        max_output_tokens: {output}")
    lines.append("        enabled: true")
    return lines


def provider_lines(provider_id: str, name: str, base_url: str, default_model: str,
                   models: list[list[str]]) -> list[str]:
    lines = [
        f"  - id: {json.dumps(provider_id)}",
        f"    display_name: {json.dumps(name)}",
        f"    base_url: {json.dumps(base_url)}",
        "    protocol: chat",
        f"    default_model: {json.dumps(default_model)}",
        "    enabled: true",
        "    models:",
    ]
    for model in models:
        lines.extend(model)
    return lines


def alibaba_provider_lines(model_id: str, display_name: str, context: int, output: int,
                           vision: bool, reasoning: bool, structured: bool) -> list[str]:
    return provider_lines(
        "alibaba", "Alibaba/Qwen", "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
        model_id,
        [model_lines(model_id, display_name, context, output, vision, True, reasoning, structured)],
    )


def desired_providers(present: set[str], proxy_url: str, proxy_models: list[str]) -> dict[str, list[str]]:
    providers: dict[str, list[str]] = {}
    if proxy_models:
        models = [model_lines(model, model, None, None, False, True, False, False) for model in proxy_models]
        providers["local-openai-proxy"] = provider_lines(
            "local-openai-proxy", "Local OpenAI Proxy", proxy_url.rstrip("/"), proxy_models[0], models
        )
    if "KIMI_API_KEY" in present:
        providers["kimi-for-coding"] = provider_lines(
            "kimi-for-coding", "Kimi For Coding", "https://api.kimi.com/coding/v1", "k3",
            [model_lines("k3", "Kimi K3", 1_048_576, 131_072, True, True, True, True)],
        )
    if "MINIMAX_API_KEY" in present:
        providers["minimax"] = provider_lines(
            "minimax", "MiniMax", "https://api.minimax.io/v1", "MiniMax-M3",
            [model_lines("MiniMax-M3", "MiniMax M3", 1_000_000, 128_000, True, True, True, False)],
        )
    if "DASHSCOPE_API_KEY" in present:
        providers["alibaba"] = alibaba_provider_lines(
            QWEN_MODEL_ID, QWEN_MODEL_NAME, 1_000_000, 131_072, True, True, True,
        )
    if "ZAI_API_KEY" in present:
        providers["zai"] = provider_lines(
            "zai", "Z.AI", "https://api.z.ai/api/paas/v4", "glm-5.2",
            [
                model_lines("glm-4.7", "GLM-4.7", 204_800, 131_072, False, True, True, False),
                model_lines("glm-5.2", "GLM-5.2", 1_000_000, 131_072, False, True, True, True),
            ],
        )
    if "MOONSHOT_API_KEY" in present:
        providers["moonshotai"] = provider_lines(
            "moonshotai", "Moonshot AI", "https://api.moonshot.ai/v1", "kimi-k2.5",
            [
                model_lines("kimi-k2.5", "Kimi K2.5", 262_144, 262_144, True, True, True, True),
                model_lines("kimi-k3", "Kimi K3", 1_048_576, 131_072, True, True, True, True),
            ],
        )
    return providers


def section_bounds(lines: list[str], key: str) -> tuple[int, int] | None:
    start = next((index for index, line in enumerate(lines) if re.match(rf"^{re.escape(key)}\s*:", line)), None)
    if start is None:
        return None
    end = len(lines)
    for index in range(start + 1, len(lines)):
        if lines[index] and not lines[index][0].isspace() and not lines[index].lstrip().startswith("#"):
            end = index
            break
    return start, end


def merge_server(lines: list[str]) -> None:
    bounds = section_bounds(lines, "server")
    defaults = [("host", '"127.0.0.1"'), ("port", "1906"), ("grpc_port", "50051")]
    if bounds is None:
        if lines and lines[-1] != "":
            lines.append("")
        lines.extend(["server:", *[f"  {key}: {value}" for key, value in defaults]])
        return
    start, end = bounds
    if lines[start].strip() != "server:":
        raise ValueError("top-level server must be a block mapping")
    present = {
        match.group(1)
        for line in lines[start + 1:end]
        if (match := re.match(r"^  ([A-Za-z0-9_]+)\s*:", line))
    }
    additions = [f"  {key}: {value}" for key, value in defaults if key not in present]
    lines[end:end] = additions


def merge_alibaba_default(lines: list[str], present: set[str]) -> None:
    if "DASHSCOPE_API_KEY" not in present:
        return

    bounds = section_bounds(lines, "llm")
    if bounds is None:
        if lines and lines[-1] != "":
            lines.append("")
        lines.extend(
            [
                "llm:",
                f"  model: {json.dumps(f'alibaba/{QWEN_MODEL_ID}')}",
                '  api_key_env: "DASHSCOPE_API_KEY"',
            ]
        )
        return

    start, end = bounds
    if lines[start].strip() != "llm:":
        raise ValueError("top-level llm must be a block mapping")
    for index in range(start + 1, end):
        if re.fullmatch(r"  model:\s*[\"']?alibaba/qwen3\.7-max[\"']?\s*", lines[index]):
            lines[index] = f"  model: {json.dumps(f'alibaba/{QWEN_MODEL_ID}')}"
        elif re.fullmatch(r"  api_key_env:\s*[\"']?QWEN_TOKENPLAN_API_KEY[\"']?\s*", lines[index]):
            lines[index] = '  api_key_env: "DASHSCOPE_API_KEY"'


def migrate_phase_alibaba_seed(lines: list[str]) -> None:
    bounds = section_bounds(lines, "providers")
    if bounds is None:
        return
    start, end = bounds
    legacy = alibaba_provider_lines(
        "qwen3-coder-plus", "Qwen3 Coder Plus", 1_048_576, 65_536, False, False, False,
    )
    replacement = alibaba_provider_lines(
        QWEN_MODEL_ID, QWEN_MODEL_NAME, 1_000_000, 131_072, True, True, True,
    )
    provider_starts = [
        index
        for index in range(start + 1, end)
        if re.match(r"^  - id:\s*", lines[index])
    ]
    provider_starts.append(end)
    for position in range(len(provider_starts) - 1):
        block_start = provider_starts[position]
        block_end = provider_starts[position + 1]
        if lines[block_start:block_end] == legacy:
            lines[block_start:block_end] = replacement
            return


def merge_provider_section(lines: list[str], providers: dict[str, list[str]]) -> None:
    if not providers:
        return
    bounds = section_bounds(lines, "providers")
    if bounds is None:
        if lines and lines[-1] != "":
            lines.append("")
        lines.append("providers:")
        start = len(lines) - 1
        end = len(lines)
    else:
        start, end = bounds
        value = lines[start].split(":", 1)[1].strip()
        if value == "[]":
            lines[start] = "providers:"
        elif value:
            raise ValueError("top-level providers must be a block sequence or []")

    existing = {
        match.group(1) or match.group(2) or match.group(3)
        for line in lines[start + 1:end]
        if (match := re.match(r"^  - id:\s*(?:\"([^\"]+)\"|'([^']+)'|([^#\s]+))", line))
    }
    additions: list[str] = []
    for provider_id, block in providers.items():
        if provider_id not in existing:
            additions.extend(block)
    lines[end:end] = additions


def atomic_write(path: Path, content: str) -> None:
    mode = stat.S_IMODE(path.stat().st_mode)
    descriptor, temporary_name = tempfile.mkstemp(prefix=".uar-config.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(content)
            handle.flush()
            os.fsync(handle.fileno())
        os.chmod(temporary_name, mode)
        os.replace(temporary_name, path)
    except BaseException:
        try:
            os.unlink(temporary_name)
        except FileNotFoundError:
            pass
        raise


def main() -> int:
    args = parse_args()
    lines = args.config.read_text(encoding="utf-8-sig").splitlines()
    merge_server(lines)
    present = present_provider_variables(args.env_file)
    proxy_models = discover_proxy_models(args.proxy_url)
    merge_alibaba_default(lines, present)
    migrate_phase_alibaba_seed(lines)
    merge_provider_section(lines, desired_providers(present, args.proxy_url, proxy_models))
    atomic_write(args.config, "\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f"native provider merge failed: {error}", file=os.sys.stderr)
        raise SystemExit(1)
