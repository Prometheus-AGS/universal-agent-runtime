# Function-first same-image supervisor preparation

Date: 2026-10-17. Child Execute FC4. **No command in this document has run.**
The operator verification gate remains CLOSED. This is a source-derived packet
for rendering exact FC6 commands after the whole-phase FC5 inventory, followed
by separate instructions for each command. It does not authorize a Docker,
Lima, controller, service, probe, build, or acceptance action.

## Frozen inputs and private outputs

FC5 must supply the actual source and fixture trees, full source commit, local
manifest and inventory digest, existing disposable guest and its private
`LIMA_HOME` and `limactl` path. FC6 must supply the inspected image ID, image
preparation receipt, retained qualified-controller record/digest, current
controller configuration and journal access, runtime/service configuration,
scoped credentials, live endpoint, provider trace and fresh output paths. None
of the old rc04–rc07 identities is a substitute for these observed values.

For the local candidate the selected image label is
`uar:1.1.1-function-first` on `linux/arm64`. The proposed build and inspection
commands from the accepted child plan are exactly:

```text
docker buildx build --load --platform linux/arm64 --tag uar:1.1.1-function-first --file Dockerfile .
docker image inspect uar:1.1.1-function-first
```

They require separate exact operator instructions after FC5. Record exit code,
the inspected `Id`, `Os`, `Architecture`, source/fixture trees and the actual
immutable preparation receipt. Its JSON fields are `schema_version: 1`,
`image`, `platform`, `source_tree`, `fixture_tree`, `image_id`. The manifest's
`prepared_image_id` must equal the inspected ID and its
`image_preparation_receipt` must hold the new receipt's absolute path and
`sha256:` digest. The functional runner inspects these same installed bytes;
it neither builds nor pulls in prebuilt mode.

## Controller and guest sequence

1. Read the retained `production_qualified: true` qualification record and
   compare its controller ID, qualification ID, mechanism and controller source
   SHA-256 with the current `scripts/supervisor/controller.py` and actual
   controller journal registration. The reviewed rc04 record is historical
   authority for the isolated native mechanism only; verify that the current
   normal controller selects it. A new image identity file cannot qualify a
   controller. A mismatch stops preparation.
2. Export the exact inspected image ID's executable and `/opt/uar` assets to
   a new private directory, install them in the existing disposable
   `dot17-linux-qualification` Lima guest, and compare the installed executable
   digest with the exported bytes. The previous private
   `install-rc05-image.py` is hard-bound to the rc05 tag and must not be invoked
   for this candidate. Its source lines 143–194 specify the existing
   `docker container create`/`docker cp`, `limactl copy`, guest `sudo install`,
   asset copy/chown and installed-hash sequence; render it with the actual FC6
   image ID, fresh export paths and guest stage. Preserve its command outcomes
   in a new private installation receipt. This operation is separately
   authorized and is not a product gate.

   The following is the source-backed command sequence to render after FC6
   supplies the actual image ID and new private paths. It is **not an
   executable approval packet** while those variables are unbound. The
   existing rc05 installer source records each command's exit status and
   cleans its owned export container; the rendered sequence must do the same.

   ```text
   docker container create --name <fresh-export-container> --platform linux/arm64 <actual-prepared-image-id>
   docker cp <fresh-export-container>:/usr/local/bin/universal-agent-runtime <fresh-export-root>/bin/universal-agent-runtime
   docker cp <fresh-export-container>:/opt/uar/. <fresh-export-root>/assets
   docker container rm --volumes <fresh-export-container>
   LIMA_HOME=<actual-private-lima-home> <actual-limactl> --tty=false copy --backend=scp --recursive <fresh-export-root> dot17-linux-qualification:<fresh-guest-stage>
   LIMA_HOME=<actual-private-lima-home> <actual-limactl> --tty=false shell --workdir=/tmp dot17-linux-qualification sudo install -o root -g root -m 755 <fresh-guest-stage>/bin/universal-agent-runtime /opt/uar/bin/universal-agent-runtime
   LIMA_HOME=<actual-private-lima-home> <actual-limactl> --tty=false shell --workdir=/tmp dot17-linux-qualification sudo cp -R <fresh-guest-stage>/assets/. /opt/uar/
   LIMA_HOME=<actual-private-lima-home> <actual-limactl> --tty=false shell --workdir=/tmp dot17-linux-qualification sudo chown -R uar:uar /opt/uar/static /opt/uar/skills /opt/uar/models /opt/uar/policies
   ```

   Compare SHA-256 of the exported and installed executable using the exact
   guest hash operation from `install-rc05-image.py:187-194`; retain both
   values and the image ID in the private install receipt. No helper should
   silently substitute an old rc05 export or manifest.
3. Start the normal independent `uar-supervisor.service` before `uar.service`
   in that guest, using the existing private mTLS and durable journal settings
   from `deploy/supervisor/` and the retained qualification digest. Start the
   candidate UAR service using the installed image bytes and existing durable
   stores. Retain its actual controller `domains.registration`, live
   capabilities/execution-owner responses and boot ID. Do not select the
   qualification-candidate flag for the retained production-qualified record.

   The service command shapes are below. Render the actual Lima executable,
   home, guest and current private configuration in FC6; request each exact
   command separately. The checked-in `uar-supervisor.service` starts the
   controller without the candidate flag.

   ```text
   LIMA_HOME=<actual-private-lima-home> <actual-limactl> --tty=false shell dot17-linux-qualification sudo systemctl start uar-supervisor.service
   LIMA_HOME=<actual-private-lima-home> <actual-limactl> --tty=false shell dot17-linux-qualification sudo systemctl start uar.service
   ```
4. The existing private
   `/Users/gqadonis/.local/state/uar-v1.1.1/supervisor/stage-qualified-rc05.mjs`
   has a generic API staging body but enforces an `rc05-NN` suffix and a
   `mode=complete, delay=601` provider mode. Its seven confirmed arguments are
   `--runtime-url`, `--credentials`, `--preparation`, `--provider-mode`,
   `--receipt`, `--repo`, `--suffix`. If used, render a fresh suffix and new
   receipt path, verify its live preparation belongs to this installed image
   and predecessor, and explicitly approve its exact `node` command. It
   installs a real package/binding/team, admits one queued and one active
   attempt, and records their source IDs/fence. It does not install UAR, boot
   the guest, inspect the provider trace, create the replacement or seal the
   identity. Old rc07 variants are hard-bound to frozen rc07 manifests and
   must not be reused.

   The proposed staging argv, to render with a fresh observed preparation
   record and endpoint, is:

   ```text
   node /Users/gqadonis/.local/state/uar-v1.1.1/supervisor/stage-qualified-rc05.mjs --runtime-url <current-loopback-origin> --credentials <private-credentials-json> --preparation <fresh-qualified-predecessor-json> --provider-mode <current-mode-complete-delay-601-json> --receipt <new-private-staging-receipt> --repo /Volumes/T7/Projects/prometheus/universal-agent-runtime --suffix <new-suffix-matching-qualified-supervisor-rc05-NN>
   ```

   This old helper is an optional staging implementation, not an assertion
   that its current private inputs are valid for this candidate. Its exact
   proposed argv must be inspected again and separately authorized before use.
5. Read the provider trace for actual active predecessor contact; retain the
   queued and active attempt IDs, team/workspace IDs, predecessor claim/fence,
   controller registration, machine ID and preboot boot ID. The guest-only
   physical transition is the previously observed `limactl stop --force
   dot17-linux-qualification`, followed by `limactl start
   dot17-linux-qualification`. Render the actual `limactl` executable and
   `LIMA_HOME`, request each exact command separately, and retain the physical
   stop/start receipts. Never reboot the host or a customer machine. A process
   restart alone does not establish native boot exclusion.

   ```text
   LIMA_HOME=<actual-private-lima-home> <actual-limactl> stop --force dot17-linux-qualification
   LIMA_HOME=<actual-private-lima-home> <actual-limactl> start dot17-linux-qualification
   ```
6. Start the same installed UAR bytes as a fresh replacement with the same
   durable stores and qualified controller, retain the changed guest boot ID,
   current journal and distinct replacement fence. The final private identity
   receipt must include phase, `isolated: true`, exact image/platform,
   source/fixture trees, `runtime_image_id`, runtime URL, actual predecessor
   registration/fence, replacement fence, and nonempty `team_scopes` with the
   queued and uncertain attempt IDs. Seal its digest into the private
   infrastructure `supervisor` fields with the qualification digest and
   read-only journal transport. Do not manufacture rows or relabel old IDs.

The recovery adapter requests and consumes its generation-1 controller
challenge during the later authorized functional command. It requires a real
changed whole-guest boot, exact predecessor/replacement registration and proof,
and preserves uncertain effects. Each affected rerun needs a new predecessor,
replacement, physical transition and unused proof. Historical rc04–rc07
reclaims are consumed and provide no new acceptance credit.

## Published children

After authorized P1 returns actual immutable arm64 and amd64 child digests,
render a separate `docker pull --platform <platform> <actual child>` and
`docker image inspect <actual child>` pair for each child and request exact
instructions. Each gets a distinct preparation receipt, matching installed
guest bytes, fresh queued/active predecessor, separate physical boot and
identity/proof. Local or other-architecture preparation does not qualify it.
The published functional command retains registry/index/tag/workflow
provenance and runs only `streams,recovery` against the already present child.

## Uncomfortable limit

The current private install helpers are tied to old candidate IDs. The
operator must render an exact new image export/install command sequence and
provide current guest/controller/service paths before FC6 can execute; this
document cannot assert they exist now. If those inputs or genuine retained
qualification are missing, preparation and certification remain blocked. The
source-backed API staging helper and an inspected image ID alone do not prove
physical exclusion.
