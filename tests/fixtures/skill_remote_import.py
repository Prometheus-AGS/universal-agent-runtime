"""External-runtime fixture for remote SKILL.md import acceptance.

The caller supplies an authenticated HTTP request callback and the lifecycle
restart callback for a running image. This module does not start a local server,
write a database, or execute an operator command.
"""
import hashlib
from pathlib import Path
from urllib.parse import quote
import uuid


FIXTURES = Path(__file__).with_name("skill-upload")
IMPORT_PATH = "/api/uar/skills/import-content"


def _require(condition, message):
    if not condition:
        raise AssertionError(message)


def _base(base_url):
    value = base_url.rstrip("/")
    _require(value.startswith(("http://", "https://")), "skill fixture requires an HTTP image base URL")
    return value


def _read(name):
    return (FIXTURES / name).read_bytes().decode("utf-8")


def _named(source, name):
    return source.replace("name: Remote Receipt", "name: " + name, 1)


def _digest(markdown):
    return "sha256:" + hashlib.sha256(markdown.encode("utf-8")).hexdigest()


def _skill_id(name):
    return name.strip().lower().replace(" ", "-")


def exercise(base_url, request, restart):
    """Upload, read, bind, restart, and re-read one manifest from an image.

    `request(method, absolute_url, body, expected_status)` must make an
    authenticated request and return a decoded JSON response. `restart()` must
    restart that same image with its persisted data still attached.
    """
    base = _base(base_url)
    suffix = uuid.uuid4().hex
    name = "Remote Receipt " + suffix
    raw = _named(_read("round-trip.SKILL.md"), name)
    expected_id = _skill_id(name)
    expected_digest = _digest(raw)

    created = request("POST", base + IMPORT_PATH, {"skill_markdown": raw}, 201)
    _require(created["skill_id"] == expected_id, "import returned a different skill ID")
    _require(created["skill_markdown"] == raw, "import response changed submitted SKILL.md bytes")
    _require(created["artifact_digest"] == expected_digest, "import digest is not SHA-256 of submitted SKILL.md")

    skill_url = base + "/api/uar/skills/" + quote(expected_id, safe="")
    readback = request("GET", skill_url, None, 200)
    _require(readback["skill_markdown"] == raw, "GET changed persisted SKILL.md bytes")
    _require(readback["artifact_digest"] == expected_digest, "GET changed persisted SKILL.md digest")

    agent_id = "release-skill-fixture-" + suffix
    agent_url = base + "/api/uar/agents/" + quote(agent_id, safe="") + "/skills"
    request("PUT", agent_url, {"skill_ids": [expected_id]}, 200)
    _require(expected_id in request("GET", agent_url, None, 200), "agent binding did not retain imported skill")

    collision = _named(_read("collision.SKILL.md"), name)
    collision_reply = request("POST", base + IMPORT_PATH, {"skill_markdown": collision}, 409)
    _require(collision_reply.get("error") == "skill_id_conflict", "collision returned the wrong error")
    unchanged = request("GET", skill_url, None, 200)
    _require(unchanged["skill_markdown"] == raw and unchanged["artifact_digest"] == expected_digest,
             "collision replaced the first stored SKILL.md")

    invalid = request("POST", base + IMPORT_PATH, {"skill_markdown": _read("invalid.SKILL.md")}, 400)
    _require(invalid.get("error") == "skill_markdown_invalid", "invalid manifest returned the wrong error")

    restart()
    after_restart = request("GET", skill_url, None, 200)
    _require(after_restart["skill_markdown"] == raw, "restart lost or changed SKILL.md bytes")
    _require(after_restart["artifact_digest"] == expected_digest, "restart changed SKILL.md digest")
    _require(expected_id in request("GET", agent_url, None, 200), "restart lost agent binding")
    return {"skill_id": expected_id, "artifact_digest": expected_digest, "agent_id": agent_id}


def exercise_persistence_failure(base_url, request):
    """Require a configured failing persistence image to reject and not publish.

    The caller points `base_url` at an image whose persistence adapter is
    intentionally unavailable before invoking this function.
    """
    base = _base(base_url)
    name = "Persistence Failure " + uuid.uuid4().hex
    raw = _named(_read("round-trip.SKILL.md"), name)
    skill_id = _skill_id(name)
    failed = request("POST", base + IMPORT_PATH, {"skill_markdown": raw}, 503)
    _require(failed.get("error") in ("skill_persistence_unavailable", "skill_persistence_failed"),
             "failed persistence returned a successful or unrelated import outcome")
    request("GET", base + "/api/uar/skills/" + quote(skill_id, safe=""), None, 404)
    return {"skill_id": skill_id, "error": failed["error"]}
