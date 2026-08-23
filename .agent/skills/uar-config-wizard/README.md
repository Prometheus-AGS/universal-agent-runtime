# UAR Configuration Wizard

> **Current authority:** [UAR configuration guide](/docs/configuration). This
> README describes the checked-in assistant assets; generated output still must
> be reviewed against the runtime configuration schema.

This skill package contains prompts, templates, state scripts, and specialist
agent instructions for helping an operator draft UAR configuration. It can
produce candidate YAML, environment templates, migration suggestions,
Kubernetes Secret/ConfigMap material, and local-model configuration. It is not
the runtime parser and cannot certify a deployment.

## Entry points

| Command | Purpose |
|---|---|
| `/uar-config` | Route a configuration request to the appropriate mode |
| `/uar-wizard` | Gather requirements for a first configuration draft |
| `/uar-validate` | Compare existing files with the skill's checked-in rules |
| `/uar-migrate` | Draft legacy environment-variable migrations |
| `/uar-k8s-config` | Draft Kubernetes Secret and ConfigMap resources |
| `/uar-model-select` | Research and compare model candidates for named hardware |
| `/uar-stack` | Draft a coordinated UAR and local-model configuration bundle |

Model research depends on the retrieval tools available to the active agent and
must cite current sources. Hardware fit, provider support, and generated
TurboQuant settings remain recommendations until checked against the selected
model server and actual hardware.

## Checked-in assets

- `assets/templates/` contains configuration, environment, Kubernetes, and
  launch-script templates.
- `references/` contains the skill's configuration notes and JSON schemas.
- `scripts/` owns session state, checkpointing, dispatch, and local validation.
- `prompts/` and `agents/` define the advisory workflow.

Named sessions are stored under `.config-wizard/sessions/<name>/` with current
state, checkpoints, output, and history. These files may contain generated
secrets or endpoints and must not be committed without operator review.

The runtime's own configuration parser, `example.config.yaml`, deployment
guides, and profile-specific verification remain authoritative when a generated
candidate disagrees with this skill package.
