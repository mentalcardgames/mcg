
### Prerequisites

- Rust 1.70+  
- Node.js 18+ (for building VS Code extension)  
- VS Code (latest stable release)  

---

### Running the VS Code Extension

> **Important:** The `.cgdsl` file must be inside a **workspace folder** so that outputs (`output.json`, `output.dot`, `output.png`) can be created.

Create a workspace folder for your game and add a `.cgdsl` file:

```bash
mkdir ~/my_cgdl_game
cd ~/my_cgdl_game
touch my_game.cgdsl
```

### Run Extension
```bash
cd cgdsl
cargo build --workspace && code --extensionDevelopmentPath=$(pwd) .
```

### Build Project

```bash
cargo build
```

## License

This project is dual-licensed under the MIT License and the Apache License (Version 2.0).

* [Apache License, Version 2.0](LICENSE-APACHE)
* [MIT License](LICENSE-MIT)
