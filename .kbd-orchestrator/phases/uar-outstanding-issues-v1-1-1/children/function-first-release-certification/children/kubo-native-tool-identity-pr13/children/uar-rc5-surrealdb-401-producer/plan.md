# UAR 1.4.0-rc.5 producer for PR #13

## Assess

The official RC.5 tag is `1f765cb9b28b34de40e3c980ce0913f4f4657fcf`. Its published image embeds SurrealDB 4.0.0. The operator requires 4.0.1 and kept RC.5 as the package version. The existing release also cannot establish that the required native tool identity survives cold resume. The earlier cumulative validation method is superseded. Its evidence remains historical.

## Analyze

The candidate starts from the exact tag and carries the 4.0.1 embedded dependency, lifecycle and fresh-state paths. The operator adopted a Cargo.lock fingerprint in UAR's existing A2A discovery card so Kubo can compare the exact version and fingerprint to the qualified immutable artifact record. The card reports dependency identity. Image provenance binds the released artifact to the candidate.

One real UAR run can exercise native tool identity and durable state through the normal owner-scoped HTTP API. A local scripted model returns one native echo call and holds continuation until the first runtime stops. The replacement process opens the same fresh 4.0.1 store, resumes the exact checkpoint, and must send the original correlated tool call and result to the model before a terminal response. Count tool execution from the real tool body. The external local identity gateway provides a stable owner outside the Cucumber steps.

## Plan

Change `uar-rc5-surreal-401-qualify` owns these tasks.

1. Integrate the isolated RC.5 source, exact 4.0.1 dependencies, fresh storage and the optional discovery fingerprint.
2. Integrate one standalone cold-restart scenario, its owned process cleanup and the dedicated loopback inputs. Record the exact candidate source and configuration.
3. After exact operator approval, build the locked `server-full,postgres-backend` release binary and execute the one scenario. Stop at failure or source drift. Require the two exact build observations, the durable checkpoint, the original call ID, one actual invocation, the expected terminal response and clean exit and cleanup evidence.
4. After separate publication authorization, bind the producer result to the actual release commit, tag, image index and platform provenance. Kubo adopts that artifact in its subsequent consumer boundary.

The port and scenario source binding is [source.json](/private/tmp/uar-rc5-native-restart-bzluj3ip/source.json). The baseline, scope and exit criteria are recorded in the [active Kubo amendment](/Volumes/T7/Projects/references/kubo/openspec/changes/uar-1-3-1-successor-pin/rc5-retarget.md). The [scenario requirements](/Volumes/T7/Projects/references/kubo/openspec/changes/uar-1-3-1-successor-pin/producer-boundary.md#standalone-scenario-source) describe the one attempt. The broader parent, installed services, old stores and existing Kubo pin remain outside this producer attempt.
