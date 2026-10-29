//! Host fixture tool: run SurrealQL from stdin against a stopped UAR embedded
//! store with the same pinned SurrealDB SDK that UAR links, then print each
//! statement's result as one JSON line. The host CLI cannot read stores written
//! by this SDK revision, so fixtures that must edit a stopped store use this.
//!
//! Usage: `surrealql_store_runner <endpoint> <namespace> <database> < query.surql`
//! where `<endpoint>` is, for example, `surrealkv:///path/to/memory.db`.
use anyhow::{Context, Result};
use std::io::Read;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let endpoint = args.next().context("missing <endpoint>")?;
    let namespace = args.next().context("missing <namespace>")?;
    let database = args.next().context("missing <database>")?;
    let mut query = String::new();
    std::io::stdin().read_to_string(&mut query).context("failed to read SurrealQL from stdin")?;

    let db = surrealdb::engine::any::connect(endpoint).await.context("failed to open the store")?;
    let outcome = run(&db, &namespace, &database, query).await;
    // Dropping the handle cannot await engine finalization; close it explicitly.
    let closed = db.shutdown().await.context("failed to shut the store down");
    outcome.and(closed)
}

async fn run(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    namespace: &str,
    database: &str,
    query: String,
) -> Result<()> {
    db.use_ns(namespace).use_db(database).await?;
    let mut response = db.query(query).await?.check()?;
    for index in 0..response.num_statements() {
        let value: surrealdb::types::Value = response.take(index)?;
        println!("{}", value.into_json_value());
    }
    Ok(())
}
