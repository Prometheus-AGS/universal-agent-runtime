## 1. Fix + test

- [x] 1.1 Apply `cli.port -> server.port` and `cli.jwt_required ->
      security.jwt_required` in `AppConfig::load_with_cli`.
- [x] 1.2 Regression tests: `cli_port_overrides_server_port`,
      `cli_jwt_required_overrides_security_jwt_required` (both green; lib 392).
- [x] 1.3 `.env.example`: note the short `PORT`/`JWT_REQUIRED` forms are honored.

## 2. Bookkeeping

- [x] 2.1 Commit, push, archive; update phase state.
