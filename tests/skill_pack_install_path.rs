#![cfg(unix)]

use std::{collections::HashSet, os::unix::fs::PermissionsExt, process::Command, sync::Arc};

use axum_test::TestServer;
use serde_json::Value;
use universal_agent_runtime::uar::{
    api::skills::build_router,
    runtime::skills::{
        builtin_loader::discover_builtin_skills, pack_detection::PackSource, service::SkillService,
    },
};

#[tokio::test]
async fn clean_prefix_install_exposes_default_pack_inventory_through_api() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let temp = tempfile::tempdir().expect("temporary clean installation prefix");
    let fake_cargo = temp.path().join("fake-cargo");
    std::fs::write(
        &fake_cargo,
        r#"#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$CARGO_TARGET_DIR/release"
cat >"$CARGO_TARGET_DIR/release/prometheus" <<'BIN'
#!/usr/bin/env bash
printf 'prometheus test binary\n'
BIN
chmod +x "$CARGO_TARGET_DIR/release/prometheus"
"#,
    )
    .expect("fake cargo is written");
    let mut permissions = std::fs::metadata(&fake_cargo)
        .expect("fake cargo metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&fake_cargo, permissions).expect("fake cargo is executable");

    let home = temp.path().join("home");
    let prefix = home.join(".config/uar/skills");
    let output = Command::new(root.join("scripts/install-uar-skill-pack.sh"))
        .arg("--source-dir")
        .arg(root.join("crates/prometheus-skill-system"))
        .arg("--prefix")
        .arg(&prefix)
        .env("CARGO", &fake_cargo)
        .output()
        .expect("installer starts");
    assert!(
        output.status.success(),
        "installer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let original_home = std::env::var_os("HOME");
    let original_sibling = std::env::var_os("PROMETHEUS_SKILL_SYSTEM_DIR");
    let original_override = std::env::var_os("UAR_BUILTIN_SKILLS_DIR");
    let original_imported = std::env::var_os("UAR_LOAD_IMPORTED_SKILLS");
    unsafe {
        std::env::set_var("HOME", &home);
        std::env::set_var(
            "PROMETHEUS_SKILL_SYSTEM_DIR",
            temp.path().join("no-sibling-checkout"),
        );
        std::env::remove_var("UAR_BUILTIN_SKILLS_DIR");
        std::env::remove_var("UAR_LOAD_IMPORTED_SKILLS");
    }

    let (skills, provenance) = discover_builtin_skills();
    println!("installed_pack_inventory={}", skills.len());

    unsafe {
        match original_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        match original_sibling {
            Some(value) => std::env::set_var("PROMETHEUS_SKILL_SYSTEM_DIR", value),
            None => std::env::remove_var("PROMETHEUS_SKILL_SYSTEM_DIR"),
        }
        match original_override {
            Some(value) => std::env::set_var("UAR_BUILTIN_SKILLS_DIR", value),
            None => std::env::remove_var("UAR_BUILTIN_SKILLS_DIR"),
        }
        match original_imported {
            Some(value) => std::env::set_var("UAR_LOAD_IMPORTED_SKILLS", value),
            None => std::env::remove_var("UAR_LOAD_IMPORTED_SKILLS"),
        }
    }

    assert_eq!(provenance.source, PackSource::InstalledPlugin);
    assert_eq!(skills.len(), 147, "default loader inventory changed");
    let expected_ids = skills
        .iter()
        .map(|skill| skill.skill_id.clone())
        .collect::<HashSet<_>>();

    let service = Arc::new(SkillService::new(None, None));
    service.register_builtins(skills).await;
    let app = axum::Router::new()
        .nest("/skills", build_router())
        .with_state(service);
    let response = TestServer::new(app).get("/skills").await;
    response.assert_status_ok();
    let rows = response.json::<Vec<Value>>();
    let actual_ids = rows
        .iter()
        .filter_map(|row| row["skill_id"].as_str().map(ToOwned::to_owned))
        .collect::<HashSet<_>>();

    assert_eq!(actual_ids, expected_ids);
    assert!(rows.iter().all(|row| row["origin"] == "builtin"));
}
