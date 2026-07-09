## 1. Add the branded hero block

- [x] 1.1 Add a centered hero `<div align="center">` at the top of `README.md` with the project title, tagline, and a badge row (license, provider count, CI status via the existing `.github/workflows/ci.yml`), reusing an existing logo asset only if it fits cleanly

## 2. Refresh the mermaid diagrams

- [x] 2.1 In the Architecture Overview diagram, replace literal `\n` line breaks in node labels with `<br/>`, leaving node ids, edges, and subgraph structure unchanged
- [x] 2.2 Check the Entity Graph data-flow diagram for the same `\n` issue and normalize any occurrences to `<br/>`, preserving structure

## 3. Verify rendering and consistency

- [x] 3.1 Confirm the badge URLs resolve (CI badge points at the real workflow) and the hero title/tagline match the README header
- [x] 3.2 Sanity-check the mermaid blocks parse (no stray quotes/ids changed) and that no prose, provider counts, model IDs, or commands were altered
