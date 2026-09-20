---
outline: false
---

# Repository Components & Layout

To keep the workspace structured and decoupled,
logical components are organized into subfolders within the Git repository.
The map below outlines the file structure and how components map to internal Rust crates:

```text
mcg/ (Repository Root)
├─ docs/                   # Submodule directory targeting this documentation website
├─ crates/                 # Centralized directory for all logical crates
│  ├─ frontend/            # Web client / WASM target (egui UI, screens, ...)
│  ├─ native_mcg/          # Native Backend & connection supervisor
│  ├─ cardgame_dsl/        # Subdirectory for the Card Game DSL
│  │  ├─ cgdsl/            # VS Code extension source for the Card Game DSL
│  │  ├─ code_gen/         # Macro Definition for our Card Game Language
│  │  ├─ front_end/        # Compiler frontend and parser for the Card Game DSL
│  │  └─ lsp_server/       # Language Server Protocol (LSP) implementation for the Card Game DSL
│  ├─ engine/              # Game Engine for playing arbitrary Card Games
│  ├─ shared/              # Shared structs & enums to break cyclic dependencies
│  └─ qr_comm/             # Standalone library for QR fountain codes
├─ media/                  # Static media assets, card designs, and logos
├─ pkg/                    # Compiled WebAssembly artifacts (automatically generated)
├─ index.html              # Browser entrypoint loading the compiled WASM client
├─ justfile                # Task runner recipes (just build, just start, etc.)
├─ Cargo.toml              # Cargo workspace configuration declaring internal crates
├─ mcg.code-workspace      # Workspace file to open with VS Code
├─ sign_rust_binaries.ps1  # Helper script for Windows-specific binary signing
├─ AGENT.md                # Instructions for AI agents on how to behave
├─ README.md
└─ LICENSE
```

## Planned Directory Refactoring

* **Crates Folder Migration:**
We have scheduled a structural refactoring to migrate our core root-level crates
(`frontend/`, `native_mcg/`, `shared/`) into the centralized `/crates/` directory.
This isolates compiler outputs, simplifies workspace setup, and removes root-level pollution.
* **Documentation Relocation:**
The `/docs` directory inside the core gameplay repository is slated for replacement.
All comprehensive documentation is now managed in this dedicated Git repository.
* **DSL Components Consolidation:**
To better organize our domain-specific language tooling,
we plan to group the related crates (`cgdsl/`, `code_gen/`, `front_end/`, and `lsp_server/`)
under a single nested subdirectory: `/crates/cardgame_dsl/`.
This separates compiler, parser, LSP, and IDE extension logic from runtime and communication components.
