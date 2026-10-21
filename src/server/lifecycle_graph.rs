//! Host close dependencies. An edge keeps the prerequisite usable until the
//! dependant's admitted operations and finalizer have acknowledged completion.

use std::collections::HashSet;

use crate::uar::runtime::lifecycle::RuntimeSupervisor;

pub(super) async fn configure(supervisor: &RuntimeSupervisor) -> anyhow::Result<()> {
    let present: HashSet<String> = supervisor
        .resources()
        .await
        .into_iter()
        .map(|scope| scope.resource_id)
        .collect();

    // Watcher intake closes before every mutable runtime resource. Configuration
    // closes first, followed by file-ingestion intake. Both scopes retain their
    // native watcher release acknowledgements before any database-dependent
    // finalizer can become eligible. No other edge may name either watcher as a
    // prerequisite, or it would invert this first-stop barrier.
    if present.contains("configuration") {
        let dependencies = present
            .iter()
            .filter(|resource| resource.as_str() != "configuration")
            .cloned()
            .collect::<Vec<_>>();
        supervisor
            .declare_dependencies("configuration", dependencies)
            .await?;
    }
    if present.contains("ingestion-files") {
        let dependencies = present
            .iter()
            .filter(|resource| {
                !matches!(resource.as_str(), "configuration" | "ingestion-files")
            })
            .cloned()
            .collect::<Vec<_>>();
        supervisor
            .declare_dependencies("ingestion-files", dependencies)
            .await?;
    }

    // Blocking descendants of initialization retain this scope even if their
    // original async waiter fails or panics. Join them before releasing any
    // acquired service they could still be using.
    if present.contains("startup") {
        let dependencies = present.iter().filter(|resource| {
            !matches!(resource.as_str(), "configuration" | "ingestion-files" | "startup")
        }).cloned().collect::<Vec<_>>();
        supervisor.declare_dependencies("startup", dependencies).await?;
    }

    let edges: &[(&str, &[&str])] = &[
        ("realtime", &["persistence"]),
        ("persistence-access", &["realtime", "persistence"]),
        ("credential-migrations", &["persistence-access"]),
        ("memory-realtime", &["memory"]),
        ("memory-expiry", &["memory-realtime", "memory"]),
        ("mcp", &["memory", "credential-migrations", "persistence-access"]),
        ("projected-mcp", &["mcp", "memory", "persistence-access"]),
        ("execution-owner", &["persistence-access", "realtime"]),
        ("run-history", &["execution-owner", "persistence-access"]),
        ("ingestion", &["persistence-access", "local-embedding"]),
        ("run-manager", &["run-history", "execution-owner", "ingestion", "projected-mcp", "mcp", "memory", "persistence-access", "local-embedding"]),
        ("actors", &["run-manager", "run-history", "projected-mcp", "memory", "persistence-access"]),
        ("sandboxes", &["actors", "run-manager", "projected-mcp", "mcp", "persistence-access"]),
        ("team-execution", &["sandboxes", "actors", "run-manager", "execution-owner"]),
        ("observers", &["team-execution", "sandboxes", "run-manager", "ingestion"]),
        ("committed-work", &["observers", "run-manager", "ingestion", "realtime", "memory-realtime"]),
        ("http", &["committed-work", "observers", "team-execution", "sandboxes", "run-manager", "credential-migrations", "ingestion", "memory", "projected-mcp", "mcp", "realtime", "memory-realtime", "local-embedding"]),
        ("a2a-grpc", &["run-manager", "team-execution", "credential-migrations", "mcp", "persistence-access"]),
    ];
    for &(resource, prerequisites) in edges {
        if present.contains(resource) {
            // Optional services have no acquired resource or close obligation.
            let dependencies = prerequisites
                .iter()
                .copied()
                .filter(|dependency| present.contains(*dependency));
            supervisor.declare_dependencies(resource, dependencies).await?;
        }
    }

    // Signal observation remains available until every other scope has
    // acknowledged closure. It is an observer, never a producer barrier.
    if present.contains("process-signals") {
        for resource in &present {
            if resource.as_str() != "process-signals" {
                supervisor
                    .declare_dependencies(resource, ["process-signals"])
                    .await?;
            }
        }
    }
    Ok(())
}
