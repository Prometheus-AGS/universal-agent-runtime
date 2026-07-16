//! Throwaway spike: does pglite-oxide's WASIX fallback boot and serve a
//! query on x86_64-apple-darwin? See
//! openspec/changes/pglite-oxide-intel-mac-spike/ for the full context.

use std::time::Instant;

use pglite_oxide::PgliteServer;
use sqlx::Connection;

#[tokio::main]
async fn main() {
    match run().await {
        Ok((boot_ms, query_ms)) => {
            println!("PASS boot_ms={boot_ms} query_round_trip_ms={query_ms}");
        }
        Err(e) => {
            println!("FAIL error={e:#}");
            std::process::exit(1);
        }
    }
}

async fn run() -> anyhow::Result<(u128, u128)> {
    let boot_start = Instant::now();
    let server = PgliteServer::temporary_tcp()?;
    let boot_ms = boot_start.elapsed().as_millis();

    let query_start = Instant::now();
    let mut conn = sqlx::PgConnection::connect(&server.database_url()).await?;

    sqlx::query("CREATE TABLE spike (id INT)")
        .execute(&mut conn)
        .await?;
    sqlx::query("INSERT INTO spike VALUES (1)")
        .execute(&mut conn)
        .await?;
    let row: (i32,) = sqlx::query_as("SELECT id FROM spike")
        .fetch_one(&mut conn)
        .await?;
    let query_ms = query_start.elapsed().as_millis();

    anyhow::ensure!(row.0 == 1, "expected id=1, got id={}", row.0);

    server.shutdown()?;
    Ok((boot_ms, query_ms))
}
