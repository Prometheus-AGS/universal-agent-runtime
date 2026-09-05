//! Integration tests for host-owned project instruction discovery.

use std::fs;

use tempfile::TempDir;
use universal_agent_runtime::uar::runtime::project_instructions::{
    ProjectInstructions, ProjectInstructionsConfig,
};

#[test]
fn root_to_cwd_instructions_are_rendered_without_reading_above_the_project_root() {
    let temporary_parent = TempDir::new().expect("temporary parent must be created");
    let project_root = temporary_parent.path().join("project");
    let cwd = project_root.join("crates/a");
    fs::create_dir_all(&cwd).expect("project fixture directories must be created");
    fs::write(
        temporary_parent.path().join("AGENTS.md"),
        "parent instructions",
    )
    .expect("parent instructions must be written");
    fs::write(project_root.join(".git"), "gitdir: fixture").expect("root marker must be written");
    fs::write(project_root.join("AGENTS.md"), "root instructions")
        .expect("root instructions must be written");
    fs::write(cwd.join("AGENTS.md"), "subdirectory instructions")
        .expect("subdirectory instructions must be written");
    let canonical_project_root = project_root
        .canonicalize()
        .expect("project root must canonicalize");
    let canonical_cwd = cwd.canonicalize().expect("cwd must canonicalize");

    let instructions = ProjectInstructions::discover(
        ProjectInstructionsConfig {
            trusted_workspaces: vec![project_root.clone()],
            ..ProjectInstructionsConfig::default()
        },
        &cwd,
    )
    .expect("trusted project instructions must be discovered");

    assert_eq!(instructions.root(), Some(canonical_project_root.as_path()));
    assert_eq!(
        instructions.render(),
        format!(
            "Project instructions: {}\nroot instructions\n\n---\n\nProject instructions: {}\nsubdirectory instructions",
            canonical_project_root.join("AGENTS.md").display(),
            canonical_cwd.join("AGENTS.md").display()
        )
    );
    assert!(!instructions.render().contains("parent instructions"));
}

#[test]
fn override_instructions_replace_the_base_file() {
    let temporary_parent = TempDir::new().expect("temporary parent must be created");
    let project_root = temporary_parent.path().join("project");
    fs::create_dir_all(&project_root).expect("project fixture directory must be created");
    fs::write(project_root.join(".git"), "gitdir: fixture").expect("root marker must be written");
    fs::write(project_root.join("AGENTS.md"), "base instructions")
        .expect("base instructions must be written");
    fs::write(
        project_root.join("AGENTS.override.md"),
        "override instructions",
    )
    .expect("override instructions must be written");
    let canonical_project_root = project_root
        .canonicalize()
        .expect("project root must canonicalize");

    let instructions = ProjectInstructions::discover(
        ProjectInstructionsConfig {
            trusted_workspaces: vec![project_root.clone()],
            ..ProjectInstructionsConfig::default()
        },
        &project_root,
    )
    .expect("trusted project instructions must be discovered");

    assert_eq!(
        instructions.render(),
        format!(
            "Project instructions: {}\noverride instructions",
            canonical_project_root.join("AGENTS.override.md").display()
        )
    );
    assert!(!instructions.render().contains("base instructions"));
}

#[test]
fn untrusted_workspace_yields_no_instructions() {
    let workspace = TempDir::new().expect("temporary workspace must be created");
    fs::write(workspace.path().join("AGENTS.md"), "untrusted instructions")
        .expect("untrusted instructions fixture must be written");

    let instructions =
        ProjectInstructions::discover(ProjectInstructionsConfig::default(), workspace.path())
            .expect("untrusted discovery must return an empty result");

    assert_eq!(instructions.root(), None);
    assert!(instructions.files().is_empty());
    assert!(instructions.render().is_empty());
}

#[test]
fn subtree_instructions_are_loaded_only_after_a_file_in_that_subtree_is_read() {
    let workspace = TempDir::new().expect("temporary workspace must be created");
    let project_root = workspace.path().join("project");
    let subtree = project_root.join("packages/feature");
    let source_file = subtree.join("src/lib.rs");
    fs::create_dir_all(source_file.parent().expect("source fixture has a parent"))
        .expect("subtree fixture directories must be created");
    fs::write(project_root.join(".git"), "gitdir: fixture").expect("root marker must be written");
    fs::write(project_root.join("AGENTS.md"), "root instructions")
        .expect("root instructions must be written");
    fs::write(subtree.join("AGENTS.md"), "subtree instructions")
        .expect("subtree instructions must be written");
    fs::write(&source_file, "pub fn fixture() {}").expect("source fixture must be written");
    let canonical_subtree = subtree.canonicalize().expect("subtree must canonicalize");

    let mut instructions = ProjectInstructions::discover(
        ProjectInstructionsConfig {
            trusted_workspaces: vec![project_root.clone()],
            ..ProjectInstructionsConfig::default()
        },
        &project_root,
    )
    .expect("root project instructions must be discovered");

    assert_eq!(instructions.files().len(), 1);
    assert!(!instructions.render().contains("subtree instructions"));

    let added = instructions
        .on_file_read(&source_file)
        .expect("governed file read must load subtree instructions");

    assert_eq!(added.len(), 1);
    assert_eq!(added[0].path, canonical_subtree.join("AGENTS.md"));
    assert_eq!(added[0].content, "subtree instructions");
    assert!(instructions.render().contains("subtree instructions"));
    assert!(
        instructions
            .on_file_read(&source_file)
            .expect("repeated governed file read must remain valid")
            .is_empty()
    );
}
