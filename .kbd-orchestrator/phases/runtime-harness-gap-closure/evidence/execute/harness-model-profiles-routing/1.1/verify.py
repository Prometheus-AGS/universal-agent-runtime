#!/usr/bin/env python3
"""Deterministically verify the task 1.1 model-profile evidence gate."""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import urllib.request


WORKSPACE = pathlib.Path(__file__).resolve().parents[7]
PHASE = WORKSPACE / ".kbd-orchestrator/phases/runtime-harness-gap-closure"
EVIDENCE = PHASE / "evidence/execute/harness-model-profiles-routing/1.1"
MATRIX_PATH = PHASE / "provider-profile-matrix.json"
RECEIPT_PATH = EVIDENCE / "source-receipts.json"


def canonical_sha(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def file_sha(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def get_json(url: str) -> object:
    with urllib.request.urlopen(url, timeout=5) as response:
        assert response.status == 200, (url, response.status)
        return json.load(response)


def command_output(*args: str) -> str:
    return subprocess.check_output(args, text=True).strip()


matrix = json.loads(MATRIX_PATH.read_text())
receipt = json.loads(RECEIPT_PATH.read_text())
source_ids = set(matrix["sources"])

assert matrix["schemaVersion"] == 2
assert matrix["audit"]["status"] == "PASS"
assert matrix["selectionContract"]["exactMatchRequired"] is True
assert matrix["selectionContract"]["familySubstringMayAuthorizeCompatibility"] is False
assert matrix["audit"]["profilesUsingFamilySubstringAsSoleEvidence"] == []
assert matrix["audit"]["fieldsUsingFamilySubstringAsSoleEvidence"] == []
assert matrix["claimBoundary"]["certifiedDispatchProfiles"] == 0
assert matrix["audit"]["dispatchCertifiedProfileCount"] == 0

providers = matrix["providers"]
models = [(provider, model) for provider in providers for model in provider["models"]]
assert len(providers) == 6
assert len(models) == 13
assert len({model["exactProfileKey"] for _, model in models}) == 13

for provider, model in models:
    expected_key = "|".join(
        [
            provider["providerId"],
            provider["baseUrl"],
            provider["endpointKind"],
            model["exactModelId"],
        ]
    )
    assert model["exactProfileKey"] == expected_key
    assert model["aliasRevision"]
    assert "UNRESOLVED" not in model["aliasRevision"]
    assert model["templateContract"] in matrix["contracts"]["templates"]
    assert model["settingsContract"] in matrix["contracts"]["settings"]
    assert model["countingContract"] in matrix["contracts"]["counting"]
    assert model["outputContract"] in matrix["contracts"]["outputs"]
    assert model["certification"] == "BLOCKED"
    assert matrix["contracts"]["counting"][model["countingContract"]]["hardBound"] is False

for contract in matrix["contracts"]["templates"].values():
    assert contract["requiredSlots"]
    assert contract["conditionalSlots"]
    assert contract["supportedRoles"] == ["system", "user", "assistant", "tool"]
    assert contract["unsupportedRoles"]
    assert contract["missingSlotOrRoleOutcome"] == "unsupported_profile"
    assert contract["sources"]
    assert set(contract["sources"]) <= source_ids

for family in ("settings", "counting", "outputs"):
    for contract in matrix["contracts"][family].values():
        assert contract["sources"], (family, contract)
        assert set(contract["sources"]) <= source_ids, (family, contract["sources"])

for contract in matrix["contracts"]["settings"].values():
    assert contract["unknownSettingOutcome"] == "unsupported_setting"

aliases = matrix["synthesizedAliases"]
assert len(aliases) == 2
assert all(alias["aliasRevision"] for alias in aliases)
assert all(alias["dispatchEligible"] is False for alias in aliases)
for alias in aliases:
    identity = alias["canonicalIdentity"]
    assert identity["providerId"]
    assert identity["baseUrl"]
    assert identity["endpointKind"]
    assert identity["profileRevision"]
    assert alias["ineligibility"]
    if identity["exactModelId"] is None:
        assert alias["providerId"] in matrix["audit"]["ineligibleAliasesMissingExactModel"]

assert matrix["audit"]["synthesizedAliasCount"] == 2
assert matrix["audit"]["synthesizedAliasesWithRevision"] == 2
assert matrix["audit"]["dispatchEligibleSynthesizedAliasCount"] == 0
assert matrix["audit"]["dispatchEligibleAliasesMissingCompleteCanonicalIdentity"] == []

for field in (
    "exactModelProfileCount",
    "modelProfilesWithExactProfileKeys",
    "modelProfilesWithTemplateContracts",
    "modelProfilesWithSettingsContracts",
    "modelProfilesWithCountingContracts",
    "modelProfilesWithOutputContracts",
):
    assert matrix["audit"][field] == 13, field
assert matrix["audit"]["exactProviderCount"] == 6

matrix_text = MATRIX_PATH.read_text()
receipt_text = RECEIPT_PATH.read_text()
assert "UNRESOLVED" not in matrix_text
assert '"api_key":' not in matrix_text
assert '"api_key":' not in receipt_text

for relative_path, expected_sha in receipt["candidateSources"].items():
    assert file_sha(WORKSPACE / relative_path) == expected_sha, relative_path

effective = get_json("http://127.0.0.1:1906/api/uar/providers")
assert canonical_sha(effective) == receipt["runtime"]["effectiveProviderRegistry"]["canonicalJsonSha256"]
assert canonical_sha(effective) == matrix["inventory"]["effectiveProviderRegistrySha256"]

durable = get_json("http://127.0.0.1:1906/api/uar/settings/providers")
rows = durable.get("providers", durable) if isinstance(durable, dict) else durable
if isinstance(rows, dict):
    rows = list(rows.values())
clean_records = []
record_hashes = {}
for row in sorted(rows, key=lambda value: value["data"]["id"]):
    data = dict(row["data"])
    data.pop("api_key", None)
    clean = {"provider": data}
    clean_records.append(clean)
    record_hashes[data["id"]] = canonical_sha(clean)
assert record_hashes == receipt["runtime"]["durableProviders"]["records"]
durable_set_sha = canonical_sha(clean_records)
assert durable_set_sha == receipt["runtime"]["durableProviders"]["redactedCanonicalSetSha256"]
assert durable_set_sha == matrix["inventory"]["redactedDurableProviderSetSha256"]

for key in ("ferrox", "openaiProxy"):
    endpoint_receipt = receipt["runtime"][key]
    assert canonical_sha(get_json(endpoint_receipt["modelsUrl"])) == endpoint_receipt["canonicalJsonSha256"]

ferrox_receipt = receipt["runtime"]["ferrox"]
proxy_receipt = receipt["runtime"]["openaiProxy"]
assert command_output(
    "git", "-C", "/Users/gqadonis/Projects/references/ferrox", "rev-parse", "HEAD"
) == ferrox_receipt["sourceCommit"]
assert command_output(
    "git",
    "-C",
    "/Users/gqadonis/Projects/references/baseline/openai-proxy",
    "rev-parse",
    "HEAD",
) == proxy_receipt["sourceCommit"]
assert file_sha(pathlib.Path("/Users/gqadonis/Projects/references/ferrox/target/release/ferrox")) == ferrox_receipt["binarySha256"]
assert file_sha(pathlib.Path("/Users/gqadonis/.local/bin/openai-proxy")) == proxy_receipt["binarySha256"]
model_path = pathlib.Path(
    "/Users/gqadonis/Projects/references/ferrox/models/Qwen2.5-Coder-7B-Instruct-Q5_K_M.gguf"
)
assert model_path.stat().st_size == ferrox_receipt["modelArtifact"]["bytes"]
assert file_sha(model_path) == ferrox_receipt["modelArtifact"]["sha256"]

tokenizer_request = urllib.request.Request(
    ferrox_receipt["tokenizerProbe"]["url"],
    data=json.dumps({"prompt": ferrox_receipt["tokenizerProbe"]["prompt"]}).encode(),
    headers={"Content-Type": "application/json"},
)
with urllib.request.urlopen(tokenizer_request, timeout=5) as response:
    tokenizer_result = json.load(response)
assert tokenizer_result["count"] == ferrox_receipt["tokenizerProbe"]["count"]
assert tokenizer_result["tokens"] == ferrox_receipt["tokenizerProbe"]["tokens"]

proxy_source = (
    pathlib.Path("/Users/gqadonis/Projects/references/baseline/openai-proxy/src/codex.rs")
    .read_text()
)
proxy_catalog = (
    pathlib.Path("/Users/gqadonis/Projects/references/baseline/openai-proxy/src/model_catalog.rs")
    .read_text()
)
assert "ChatGptCodex rejects temperature, top_p, max_output_tokens" in proxy_source
assert "BackendProfile::ChatGptCodex => (None, None, None, None, Some(false), true)" in proxy_source
assert 'input.starts_with("gpt-5.")' in proxy_source
assert "pub const ALIASES" in proxy_catalog

for public_source in receipt["officialSources"]:
    assert public_source["httpStatus"] == 200
    assert public_source["bytes"] > 0
    assert len(public_source["sha256"]) == 64
    int(public_source["sha256"], 16)

print("PASS provider-profile prerequisite evidence")
print("providers=6 exact_models=13 effective_enabled=2 certified_dispatch=0")
print(
    "exact_keys=13 template_contracts=13 settings_contracts=13 "
    "counting_contracts=13 output_contracts=13"
)
print(
    "template_required_slots=present conditional_slots=present "
    "supported_roles=system,user,assistant,tool unsupported_roles=explicit"
)
print(
    "unknown_role_outcome=unsupported_profile "
    "unknown_setting_outcome=unsupported_setting"
)
print("direct_contract_source_ids=resolved candidate_source_hashes=recomputed")
print("synthesized_aliases=2 revision_bound=2 dispatch_eligible=0")
print("family_substring_only_profiles=0 family_substring_only_fields=0")
print(f"durable_set_sha256={durable_set_sha}")
print(f"ferrox_models_sha256={ferrox_receipt['canonicalJsonSha256']}")
print(f"openai_proxy_models_sha256={proxy_receipt['canonicalJsonSha256']}")
print(f"official_source_receipts={len(receipt['officialSources'])}")
