# Tasks: configurable-sidecar-port

- [x] 1.1 Parse the existing CLI before binding and carry it into runtime configuration loading.
- [x] 1.2 Default the preferred sidecar port to 1906 and advance across occupied ports.
- [x] 1.3 Preserve the single retained loopback listener and `READY:{effective_port}` contract.
- [ ] 1.4 Exercise preferred-port fallback through the packaged sidecar boundary with The Boss.
