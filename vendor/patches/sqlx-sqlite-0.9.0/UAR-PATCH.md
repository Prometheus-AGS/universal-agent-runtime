# SQLx SQLite 0.9.0 dependency range backport

Source is the published sqlx-sqlite 0.9.0 crate, SHA-256
`488e99c397a62007e4229aec669a179816339afc6d2620ca6fa420dbee2e982c`.
The consumer workspace owns the lockfile. Both manifests allow
libsqlite3-sys 0.39.x by applying the upper-bound changes from
[94aafe3](https://github.com/launchbadge/sqlx/commit/94aafe3a68884d923b0798a767c8d7f6cfda89d2)
and [10462fa](https://github.com/launchbadge/sqlx/commit/10462fa4414c6c7f1ed1d39bb31678e3b726878c).
The root lockfile selects the patched SQLite binding for RUSTSEC-2026-0459.
SQLx SQLite source and licenses are retained. The UAR PostgreSQL-only feature
selection is unchanged.
