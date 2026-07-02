# MCG Project Instructions

## File Editing Restriction

**Do not edit, modify, or write files outside of the `/crates/engine` directory without explicit permission from the user.**

This restriction applies to all files in the project except those within `crates/engine/`. If you need to make changes to any other part of the codebase (including but not limited to `frontend/`, `native_mcg/`, `shared/`, configuration files, documentation, or any other top-level files), you must ask the user for permission before doing so.

### Rationale

This project (`mcg`) is a Cargo workspace. While most development happens in `crates/engine`, other workspace crates may contain files that should not be modified without explicit authorization.

## Documentation-First Rule for Engine

When working on anything within `/crates/engine`, **always search the documentation in `/crates/engine/docs` before manually reading any code.**

This applies to all tasks—understanding architecture, finding code, fixing bugs, or adding features. The docs directory contains design documents, guides, and references that may answer your question faster than code exploration.
