## 1. Package and models

- [x] 1.1 Set package version 1.0.0, MIT metadata, and runtime dependencies
- [x] 1.2 Define strict Pydantic models for chat, runs, checkpoints, streaming, knowledge, and ingestion

## 2. Client surface

- [x] 2.1 Implement chat, streaming chat, tool-call, structured-output, and embedding methods
- [x] 2.2 Implement create, stream, cancel, resume, and checkpoint run methods
- [x] 2.3 Implement knowledge-base CRUD, search, document upload, and ingestion methods
- [x] 2.4 Implement typed API and transport errors plus lifecycle handling

## 3. Documentation and examples

- [x] 3.1 Add six runnable examples covering the required 1.0 surface
- [x] 3.2 Add Sphinx autodoc configuration and hosting instructions
- [x] 3.3 Update package README and breaking-change guide

## 4. Verification

- [x] 4.1 Add focused mocked HTTP/SSE contract tests
- [x] 4.2 Run Ruff, mypy, pytest, package build, and Sphinx build
- [x] 4.3 Validate this OpenSpec change with strict mode
