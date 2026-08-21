## 1. Candidate
<!-- EVIDENCE: implementation/integration is complete; requires the final immutable candidate. -->
- [ ] 1.1 Freeze the final `1.0.0` source commit under next unused candidate tag `v1.0.0-rc.3`; record source/lock/catalog digests.
- [ ] 1.2 Run complete local security, offline, platform, UI and resilience matrices.
- [ ] 1.3 Publish signed candidate artifacts and evidence manifest.
## 2. Clean installs
<!-- EVIDENCE: local candidate-certification driver implements these journeys. -->
- [ ] 2.1 Install binary/archive and container on every supported platform without a development checkout.
- [ ] 2.2 Execute docs install/config/backup/restore/upgrade/troubleshoot paths.
- [ ] 2.3 Run stable capability smoke and functional certification from artifacts.
## 3. External validation
<!-- TIME_BOUND: 3.1 requires three external installs and one real week of operation. -->
- [ ] 3.1 Record at least three external installations and one week of operation without maintainer intervention.
- [ ] 3.2 Open focused changes for every failure; rerun candidate on any source change.
- [ ] 3.3 Approve immutable evidence bundle and validate OpenSpec.
