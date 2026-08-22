# Goals — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-graceful-shutdown-deadline-semantics

- Make shutdown_timeout_secs a maximum graceful-drain deadline rather than a pre-drain delay
- Prove non-root container SIGTERM exits 0 before the outer orchestrator deadline
- Freeze a corrected candidate and rerun the complete local operational-resilience certification
