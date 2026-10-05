# Team workspace host context

## Why
C14.1 Work coding submission currently projects installed member tools but does not propagate the ordinary trusted host working directory, run MCP resources, or paired tool admission into team turns. A coding team cannot perform the requested bounded repository change without this observed prerequisite.

## What Changes
Add host-authenticated attachment to an existing scoped team instance and reuse ordinary run MCP admission and host tool authority. Keep credentials and endpoints ephemeral; keep identity, exact member resource selection and canonical workspace reference in the existing private binding. Require explicit reattachment after runtime restart before queued dispatch or recovery. Advertise the production host capability in the normal qualified team profile.

## Capabilities
- Added: team-host-context

## Impact
UAR team runtime, exact member projection and collaboration API. Boss resolves workspace selectors, pins model/profile settings, and reattaches its host bridge before recovery. No new scheduler, agent filesystem writer, workflow profile, or operation-mode override.
