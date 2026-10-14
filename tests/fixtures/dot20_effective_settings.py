"""Real HTTP acceptance helpers. Execute only at the complete Dot boundary.

The parent runner owns installation, credentials, mutations/restarts and schema
validation against the pinned Kubo schema. This module never creates a fake UAR.
"""

import json
import urllib.error
import urllib.request


def read(base_url, token=None):
    headers = {"Accept": "application/json"}
    if token is not None:
        headers["Authorization"] = "Bearer " + token
    request = urllib.request.Request(
        base_url.rstrip("/") + "/api/uar/settings/effective", headers=headers
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.status, json.load(response)
    except urllib.error.HTTPError as error:
        return error.code, error.read().decode("utf-8")


def inspect_installed(base_url, admin_token, user_token, expected, secret_markers):
    """Check real Bearer authority and operator-known effective settings."""
    for token, status in [(None, 401), ("invalid.token", 401), (user_token, 403)]:
        actual, _ = read(base_url, token)
        if actual != status:
            raise AssertionError(f"effective settings auth: expected {status}, got {actual}")
    status, body = read(base_url, admin_token)
    if status != 200:
        raise AssertionError(f"Bearer admin alone failed: {status}")
    for key in ["revision", "llm", "knowledge_base_embedder", "memory_embedder",
                "governance", "native_tools", "mcp_servers"]:
        if key not in body:
            raise AssertionError(f"missing effective field: {key}")
    for key, value in expected.items():
        if body.get(key) != value:
            raise AssertionError(f"effective setting differs from installed configuration: {key}")
    serialized = json.dumps(body, sort_keys=True)
    if any(marker and marker in serialized for marker in secret_markers):
        raise AssertionError("effective settings exposed a configured secret marker")
    again_status, again = read(base_url, admin_token)
    if again_status != 200 or again["revision"] != body["revision"]:
        raise AssertionError("unchanged effective settings did not retain their revision")
    return body


def require_effective_change(before, after, changed_field):
    """Call after an actual settings change and its documented apply boundary."""
    if before[changed_field] == after[changed_field]:
        raise AssertionError(f"{changed_field} did not change after its apply boundary")
    if before["revision"] == after["revision"]:
        raise AssertionError("effective change did not change aggregate revision")
