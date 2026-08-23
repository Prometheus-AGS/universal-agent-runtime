# Project Init TUI

> **Current authority:** [Source installation guide](/docs/installation). This
> source directory contains a template-initialization tool; it is not a UAR
> runtime installer.

The Rust source in this directory prompts for project metadata, replaces
template tokens, optionally removes unused scaffolding, and can remove its own
initializer files after a generated project is reviewed.

## Checkout status

`tools/project-init` is not a member of the root Cargo workspace and its
manifest is not declared as a nested standalone workspace. Therefore this
checkout does not currently expose a supported `cargo run` command for the
tool. The old README commands implied otherwise.

Treat the source as retained project-template tooling until a change explicitly
owns one of these choices:

1. add it to the root workspace and lockfile, or
2. make it an independently locked nested workspace.

Do not copy the source to another directory merely to bypass Cargo's workspace
boundary. That would test a different package layout from the repository.

The uncomfortable consequence is explicit: the feature list in the source is
not an availability claim while the manifest boundary remains unresolved.
