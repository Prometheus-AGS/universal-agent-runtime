# Dot17 automatic memory capture source handoff

Owned files: `scripts/dot/memory_capture_journey.py` and
`docs/dot-memory-capture-qualification.md`. Phase: Dot Execute. Source authored;
no verification executed. Runner integration remains the phase runner's file
ownership. Entrypoint: `run_memory_capture(config, evidence)` as an isolated
stage with its own evidence directory, actual child-process hosts and peers.

The driver imports the unchanged existing GuestClock. A real TLS embedding peer
holds actual requests at the native capture's pre-commit I/O boundary. Guest-only
DNS/trust, dummy credentials and real durable guest-clock storage are provisioning
prerequisites; the script cannot provision or forge them. Policy uses production
HTTP and capture uses the complete compatibility chat SSE route. No protected
storage writes, fabricated capture admission, mocked persistence, arbitrary
callback, supplied pass result or runtime clock hook is introduced.

Authored production-path assertions cover default-off actual chat, all four fixed
retention intervals and until-deleted, first-capture anchors, actual expiry at
each interval, shortening existing captures, extending only new captures,
expired identity nonrevival, queued current-policy evaluation, consent epoch
revocation after disable/re-enable, shared-reader visibility, joined restart and
explicit until-deleted removal. See the guide for exact configuration and dates.

The uncomfortable case is the fire-and-forget final save without a public queue
completion receipt. A held embedding request establishes a real transaction
boundary; sequential assistant embedding proves the first revoked save returned.
The final absence assertion is explicitly bounded and accompanied by a successful
fresh-epoch capture. No indefinite liveness or whole-boot exclusion is claimed.

All syntax, TLS trust, HTTP/stream composition, actual storage serialization,
expiry behavior, queued races, runtime cleanup and clock restoration remain
unverified. No test/check/build/lint/formatter/validator/probe/review gate ran.
No production runtime code, dependency, workflow or canonical task state changed.
