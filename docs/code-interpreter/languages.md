# UAR Code Interpreter — Language Support

_Last updated: 2026-02-21_

The code interpreter provides first-class support for four languages, each with a pre-built OCI sandbox image, correct toolchain version, default packages, and execution conventions.

---

## Language Matrix

| Language | Runtime | Execution | Package mgr | Project mode |
|---|---|---|---|---|
| **Bash** | `/bin/bash` | Direct interpreter | `apt` / manual | ✅ shell scripts + Makefiles |
| **Rust** | `rustup` + `cargo` | Compile + run | `cargo` / crates.io | ✅ full `cargo` project |
| **Python** | `python3` + `venv` | Direct interpreter | `pip` / `poetry` | ✅ `pyproject.toml` / `setup.py` |
| **Node.js** | `node` (LTS) | Direct interpreter | `npm` / `yarn` / `pnpm` | ✅ `package.json` projects |

---

## 1. Bash

### Sandbox image
```dockerfile
# build/images/bash/Dockerfile
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y \
    bash curl wget git jq unzip zip tar \
    build-essential make cmake \
    sqlite3 postgresql-client \
    python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
```

### Execution
```rust
// src/languages/bash.rs
impl LanguageConfig for Bash {
    fn execution_command(&self, request: &ExecutionRequest) -> Vec<String> {
        match request.mode {
            // Inline code → write to temp file and execute
            _ => vec!["bash".into(), "-c".into(), request.code.clone()]
        }
    }
    
    fn file_extension(&self) -> &str { "sh" }
    
    fn project_entry_command(&self, _project_path: &str) -> Option<Vec<String>> {
        // Look for Makefile, run.sh, or entrypoint.sh
        Some(vec!["bash".into(), "run.sh".into()])
    }
}
```

### Supported in all execution modes
- **Ephemeral:** `bash -c "<code>"`
- **Session:** Same session reuses environment; `cd`, `export`, and `alias` persist between turns via state injection
- **Project:** Full shell environment; reads Makefile, can run `make build`, `make test`

---

## 2. Rust

### Sandbox image
```dockerfile
# build/images/rust/Dockerfile
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y \
    curl build-essential pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
# Install rustup + stable toolchain
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:$PATH"
# Pre-warm common dependency compilations
RUN cargo install cargo-watch cargo-edit
WORKDIR /workspace
```

### Execution modes

| Mode | Command | Notes |
|---|---|---|
| Snippet | `rustc main.rs -o out && ./out` | Single-file, no Cargo |
| Project (expression) | `cargo script main.rs` | via cargo-script |
| Project (Cargo) | `cargo run` / `cargo build` / `cargo test` | Full Cargo workspace |

```rust
// src/languages/rust.rs
impl LanguageConfig for Rust {
    fn execution_command(&self, request: &ExecutionRequest) -> Vec<String> {
        if request.code.contains("fn main") && !request.code.contains("[package]") {
            // Snippet mode: wrap in a Cargo project or use rustc directly
            vec!["rust-script".into(), "main.rs".into()]
        } else {
            // Cargo project mode
            vec!["cargo".into(), "run".into()]
        }
    }
    
    fn file_extension(&self) -> &str { "rs" }
    
    fn project_entry_command(&self, _project_path: &str) -> Option<Vec<String>> {
        Some(vec!["cargo".into(), "run".into()])
    }
    
    fn project_test_command(&self) -> Option<Vec<String>> {
        Some(vec!["cargo".into(), "test".into()])
    }
}
```

### Key capabilities
- Full `cargo` (dependencies compile inside sandbox via crates.io)
- `cargo build --release` for optimized builds
- `cargo test` with streaming test output
- Cross-compilation toolchains available (optional, larger image)
- `wasm32-unknown-unknown` target available for building WASM

---

## 3. Python

### Sandbox image
```dockerfile
# build/images/python/Dockerfile
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y \
    python3 python3-pip python3-venv python3-dev \
    build-essential libssl-dev libffi-dev \
    && rm -rf /var/lib/apt/lists/*
# Pre-install common data science + utility packages
RUN pip3 install --no-cache-dir \
    numpy pandas scipy matplotlib seaborn \
    requests httpx pydantic \
    pytest black ruff
WORKDIR /workspace
```

### Execution modes

| Mode | Command | Notes |
|---|---|---|
| Snippet | `python3 -c "<code>"` | Inline expression |
| Script | `python3 main.py` | Written to file first |
| Project (pip) | `pip install -r requirements.txt && python3 main.py` | requirements.txt |
| Project (poetry) | `poetry install && poetry run python main.py` | pyproject.toml |
| Interactive | `python3` REPL over stdin/stdout | Session mode |

```rust
// src/languages/python.rs
impl LanguageConfig for Python {
    fn execution_command(&self, request: &ExecutionRequest) -> Vec<String> {
        vec!["python3".into(), "main.py".into()]
    }
    
    fn setup_commands(&self, request: &ExecutionRequest) -> Vec<Vec<String>> {
        // Check if requirements.txt exists and install if so
        vec![
            vec!["bash".into(), "-c".into(), 
                "[ -f requirements.txt ] && pip3 install -q -r requirements.txt || true".into()]
        ]
    }
    
    fn file_extension(&self) -> &str { "py" }
}
```

### Session-mode state persistence

Python session sandboxes persist the REPL state by writing variable snapshots to `/workspace/.session_state.pkl`:

```python
# Injected at session start
import dill, os
_SESSION_FILE = '/workspace/.session_state.pkl'
if os.path.exists(_SESSION_FILE):
    with open(_SESSION_FILE, 'rb') as f:
        globals().update(dill.load(f))
# ... user code runs ...
# Injected at session end
with open(_SESSION_FILE, 'wb') as f:
    dill.dump({k: v for k, v in globals().items() if not k.startswith('_')}, f)
```

---

## 4. Node.js

### Sandbox image
```dockerfile
# build/images/node/Dockerfile
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y curl build-essential git && rm -rf /var/lib/apt/lists/*
# Install Node.js LTS via nvm
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
ENV NVM_DIR="/root/.nvm"
RUN . $NVM_DIR/nvm.sh && nvm install --lts && nvm use --lts
ENV PATH="/root/.nvm/versions/node/$(ls /root/.nvm/versions/node)/bin:$PATH"
# Install common global packages
RUN npm install -g yarn pnpm tsx ts-node typescript
WORKDIR /workspace
```

### Execution modes

| Mode | Command | Notes |
|---|---|---|
| JavaScript snippet | `node -e "<code>"` | Inline |
| JavaScript file | `node main.js` | Written to file |
| TypeScript snippet | `tsx main.ts` | via tsx (no separate compile step) |
| TypeScript file | `ts-node main.ts` | Full TypeScript |
| Project (npm) | `npm install && node index.js` | package.json |
| Project (yarn) | `yarn && yarn start` | yarn workspaces supported |

```rust
// src/languages/node.rs
impl LanguageConfig for Node {
    fn execution_command(&self, request: &ExecutionRequest) -> Vec<String> {
        if request.code.contains(": ") || request.code.contains("interface ") {
            // Likely TypeScript
            vec!["tsx".into(), "main.ts".into()]
        } else {
            vec!["node".into(), "main.js".into()]
        }
    }
    
    fn file_extension(&self) -> &str { 
        // Heuristic: if code looks like TS, use .ts
        "js"  
    }
    
    fn setup_commands(&self, request: &ExecutionRequest) -> Vec<Vec<String>> {
        vec![vec!["bash".into(), "-c".into(),
            "[ -f package.json ] && npm install --silent || true".into()]]
    }
}
```

---

## 5. Universal Image (Project Mode)

For complex agent workflows that need multiple runtimes, a `universal` image is provided:

```dockerfile
# build/images/universal/Dockerfile
FROM ubuntu:24.04
# All toolchains: Rust + Python + Node + Bash utils
# ~2.5 GB image (only used for project/swarm modes where startup time is acceptable)
```

---

## 6. Language Auto-Detection

When an agent calls `code_exec` without specifying a language, the interpreter auto-detects:

```rust
pub fn detect_language(code: &str) -> Language {
    let code = code.trim();
    if code.starts_with("#!/bin/bash") || code.starts_with("#!/usr/bin/env bash") {
        return Language::Bash;
    }
    if code.contains("fn main()") || code.contains("use std::") || code.contains("println!") {
        return Language::Rust;
    }
    if code.contains("def ") || code.contains("import ") || code.contains("print(") {
        return Language::Python;
    }
    if code.contains("const ") || code.contains("require(") || code.contains("console.log") {
        return Language::Node;
    }
    // Default to Python (most AI-generated code is Python)
    Language::Python
}
```
