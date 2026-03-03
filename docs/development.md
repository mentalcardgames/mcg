
### Prerequisites

- Rust 1.70+  
- Node.js 18+ (for building VS Code extension)  
- VS Code (latest stable release)  

---

### Test Extension for Linux/macOS
```bash
# 1. Navigate into the extension directory
cd cgdsl

# 2. Install dependencies (Required to generate the grammar)
npm install

# 3. Build the Rust LSP and compile the TypeScript source
# This ensures the binary exists in ../target/debug/
npm run compile

# 4. Launch the Extension Development Host
code --extensionDevelopmentPath="$PWD" ./test-workspace
```

### Build Project

```bash
cargo build
```

## License

This project is dual-licensed under the MIT License and the Apache License (Version 2.0).

* [Apache License, Version 2.0](LICENSE-APACHE)
* [MIT License](LICENSE-MIT)
