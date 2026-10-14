#!/usr/bin/env python3
"""Deferred real-runtime HTTP fixture. Never runs during Dot implementation.

Requires operator-prepared actual native boot/Pod exclusion cases, real UAR
processes and their durable store. Does not supply fake controller observations,
forge tokens, reboot a host or infer effect settlement from HTTP success.
"""
import argparse
import concurrent.futures
import json
import os
import pathlib
import ssl
import urllib.error
import urllib.parse
import urllib.request
import uuid


def require(value, label):
    if not value:
        raise RuntimeError(label)


def request(base, path, token_env, body, admin_env=None):
    url = urllib.parse.urlsplit(base)
    require(url.scheme == "https" or (url.scheme == "http" and url.hostname in ("127.0.0.1", "localhost", "::1")), "unsafe_fixture_endpoint")
    headers = {"Authorization": "Bearer " + os.environ[token_env], "Content-Type": "application/json"}
    if admin_env:
        headers["x-uar-admin-key"] = os.environ[admin_env]
    req = urllib.request.Request(base.rstrip("/") + path, data=json.dumps(body).encode(), headers=headers, method="POST")
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *_):
            raise RuntimeError("fixture_redirect_refused")
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect(),
        urllib.request.HTTPSHandler(context=ssl.create_default_context()))
    try:
        response = opener.open(req, timeout=75)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read(2 * 1024 * 1024 + 1)
        require(len(raw) <= 2 * 1024 * 1024, "fixture_response_limit")
        value = json.loads(raw) if raw else {}
        return response.status, value


def run_case(config, case):
    base = config["runtime_urls"][0]
    host = config["host_token_env"]
    ordinary = config["ordinary_user_token_env"]
    path = "/api/uar/recovery/supervisor/challenges"
    challenge = {"runtime_epoch": case["runtime_epoch"], "target": case["target"]}
    status, _ = request(base, path, ordinary, challenge)
    require(status == 403, "ordinary_identity_obtained_host_recovery")
    wrong = json.loads(json.dumps(challenge))
    if wrong["target"]["kind"] == "checkpoint_epoch":
        wrong["target"]["runtime_epoch"] = str(uuid.uuid4())
    else:
        wrong["target"]["fence"]["incarnationId"] = str(uuid.uuid4())
    status, _ = request(base, path, host, wrong)
    require(status == 409, "wrong_incarnation_challenge_accepted")
    status, result = request(base, path, host, challenge)
    require(status == 200, "real_predecessor_challenge_unavailable")
    durable = result["challenge"]
    require(durable["target"] == case["target"] and durable["registration"]["runtime_epoch"] == case["runtime_epoch"], "challenge_identity_changed")
    exclude_path = path + "/" + durable["id"] + "/exclude"
    status, result = request(base, exclude_path, ordinary, {})
    require(status == 403, "ordinary_identity_obtained_exclusion")
    status, result = request(base, exclude_path, host, {})
    expected = 200 if case["expect_excluded"] else 409
    require(status == expected, "physical_exclusion_outcome_mismatch")
    if case["expect_excluded"]:
        require(result["exclusion_recorded"] is True and result["effects_reconciled"] is False
            and result["execution_resumed"] is False, "exclusion_overclaimed_effects")
        status, _ = request(base, exclude_path, host, {})
        require(status == 409, "challenge_replay_accepted")
    receipt = {"name": case["name"], "challenge_id": durable["id"], "exclusion_status": expected,
               "challenge_status": 200, "challenge_path": path, "exclusion_path": exclude_path,
               "ordinary_challenge_status": 403, "wrong_incarnation_status": 409,
               "ordinary_exclusion_status": 403,
               "replayed_challenge_status": 409 if case["expect_excluded"] else None}
    if "resume" in case:
        resume = case["resume"]
        body = json.loads(pathlib.Path(resume["private_body_file"]).read_text())
        resume_path = "/api/uar/runs/" + urllib.parse.quote(resume["run_id"], safe="") + "/resume/" + urllib.parse.quote(resume["checkpoint_id"], safe="")
        # Two real callers contend for the exact source claim, even with a single
        # native runtime URL. Shared profile supplies distinct runtime A/B URLs.
        urls = config["runtime_urls"]
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            futures = [pool.submit(request, urls[index % len(urls)], resume_path,
                resume["owner_token_env"], body) for index in range(2)]
            responses = [future.result() for future in futures]
            outcomes = [response[0] for response in responses]
        require(outcomes.count(200) == resume["expected_successes"], "resume_CAS_winner_count")
        require(all(status in (200, 409) for status in outcomes), "resume_unexpected_status")
        receipt["resume_statuses"] = outcomes
        receipt["resume_path"] = resume_path
        receipt["resumed_run_ids"] = [value['run_id'] for status, value in responses if status == 200]
    if "team_reclaim" in case:
        team = case["team_reclaim"]
        body = dict(team["body"])
        body["fencingEvidenceRef"] = durable["id"]
        status, result = request(base, team["path"], host, body, team["admin_key_env"])
        require(status == 200, "team_reclaim_failed")
        # The fixture supplies the exact expected IDs from real queued/running
        # predecessor work; a new empty catalog cannot satisfy this scenario.
        value = result.get("data", result)
        require(sorted(value["transferredQueuedAttemptIds"]) == sorted(team["queued_attempt_ids"]), "unsafe_team_transfer")
        require(sorted(value["uncertainAttemptIds"]) == sorted(team["uncertain_attempt_ids"]), "lost_uncertain_attempt")
        require(value["replacementFence"]["epoch"] == value["previousFence"]["epoch"] + 1, "team_epoch_not_monotonic")
        receipt["team_reclaim"] = "committed"
    return receipt


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--case-file", required=True)
    parser.add_argument("--receipt", required=True)
    args = parser.parse_args()
    config = json.loads(pathlib.Path(args.case_file).read_text())
    receipts = []
    try:
        for case in config["cases"]:
            receipts.append(run_case(config, case))
    except Exception:
        pathlib.Path(args.receipt).write_text(json.dumps({"status": "failed", "completed_cases": receipts}, indent=2) + "\n")
        raise SystemExit("supervisor production HTTP scenario failed; private inputs and responses omitted")
    pathlib.Path(args.receipt).write_text(json.dumps({"status": "http_scenarios_passed", "cases": receipts,
        "qualification": "requires independent process/domain/effect/durable-record observations"}, indent=2) + "\n")


if __name__ == "__main__":
    main()
