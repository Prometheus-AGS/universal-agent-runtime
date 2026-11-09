#[path = "certification/mod.rs"]
mod certification;

#[path = "integration/database/mod.rs"]
mod database;
#[path = "integration/live/mod.rs"]
#[expect(
    dead_code,
    reason = "this target omits the shared run_scoped, delayed-response and prompt-prefix fixtures"
)]
mod live;
#[path = "integration/services/mod.rs"]
mod services;
