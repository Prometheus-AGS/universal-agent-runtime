# ── UAR Built-in Skills ──────────────────────────────────────────────────────
#
# Skills are loaded from the ./skills/ directory by FilesystemStorageProvider.
# Each skill lives in its own subdirectory containing a SKILL.md file.
# This ConfigMap supplies those SKILL.md files so they are available without
# baking them into the container image.
#
# Mount path in container: /app/skills/<skill-name>/SKILL.md
# (Assumes the UAR binary's working directory is /app — the standard Rust Docker WORKDIR.)

resource "kubernetes_config_map" "uar_skills" {
  metadata {
    name      = "uar-skills"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "uar"
      "app.kubernetes.io/component" = "skills"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  # Each key maps to a file that will be projected into the skills volume.
  # Key format: <skill-id>---SKILL.md  (triple dash avoids slashes, which are
  # not valid in ConfigMap key names; the VolumeProjection subPath handles routing).
  data = {
    "artifact-refiner---SKILL.md" = <<-SKILL
---
name: artifact-refiner
version: "1.1.0"
description: >
  Use this skill when creating or iteratively refining named artifacts (logos, UI components,
  A2UI specifications, images, code, content, or meta-prompts) using structured PMPO orchestration with
  explicit constraints, deterministic execution, persistent artifact state, and cross-session retrieval.
authors:
  - "Travis James"
tools:
  - code_interpreter
  - file_system
  - image_generation
  - browser_renderer
triggers:
  keywords:
    - refine
    - artifact
    - logo
    - ui component
    - iterate
    - pmpo
    - improve artifact
    - refine image
    - create artifact
    - refine logo
    - refine content
    - refine code
    - a2ui
  semantic: >
    Refine, improve, or iteratively create a named artifact such as a logo, UI component,
    image, code, or content using the PMPO orchestration loop.
---

# Artifact Refiner

A PMPO-driven, artifact-centric refinement engine capable of creating and iteratively improving
artifacts across multiple domains using AI reasoning and deterministic code execution. Supports
both direct artifact output and meta-prompt refinement for generating prompts that drive other
processes.

## Supported Artifact Domains

- **Logos & brand systems** — SVG/PNG variants, wordmarks, icons, showcase pages
- **React / HTML UI concepts** — Component hierarchies, design tokens, accessibility
- **A2UI specifications** — Structural integrity, schema compliance, normalization
- **Image assets** — Composition, brand colors, resolution, format conversion
- **Content artifacts** — Markdown/HTML structure, tone, heading normalization
- **Code artifacts** — Source files in any language, lint, test, format
- **Meta-prompts** — Prompts for image/video generation, agent instructions, workflow orchestration

## Core Principles

1. **Artifact-centric** — State persisted to disk, never in conversational context
2. **Tool-augmented** — Uses code interpreter / e2b sandbox for deterministic transformations
3. **Constraint-driven** — Structured constraints with severity levels drive convergence
4. **Iterative** — Explicit convergence rules and maximum iteration guards
5. **PMPO meta-loop** — Specify → Plan → Execute → Reflect → Persist → Loop/Terminate
6. **Named & persistent** — Artifacts retrieved by name across sessions
7. **Content-type aware** — Direct output vs meta-prompt refinement with distinct evaluation strategies

## Execution Model (PMPO Loop)

### Phase Loop

1. **Specify** — Transform intent into structured specification
2. **Plan** — Convert specification into executable strategy
3. **Execute** — Apply transformations via AI + deterministic tools
4. **Reflect** — Evaluate outputs against constraints
5. **Persist** — Write validated state to disk
6. **Loop or Terminate** — Continue if constraints unsatisfied, stop if converged

## Termination Conditions

Refinement ends when:
- No blocking constraint violations remain
- All required artifact outputs exist
- Maximum iterations (5) reached

## Quick Start

Use domain-specific slash commands for focused refinement:

- `/refine-logo` — Logo and brand system refinement
- `/refine-ui` — React/HTML UI component refinement
- `/refine-content` — Content/Markdown refinement
- `/refine-image` — Image artifact refinement
- `/refine-a2ui` — A2UI specification refinement
- `/refine-status` — Check current refinement progress
- `/refine-validate` — Run validation checks on current state
SKILL
  }
}
