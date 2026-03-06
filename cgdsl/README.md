# cgdsl (Card Game Description Language)

**CGDSL** is a specialized VS Code extension for designing and visualizing card game logic. It provides a formal language to describe game rules, state transitions, and card behaviors, offering real-time feedback through visual graphs and structured data.

## 🚀 Features

* **Syntax Highlighting & Diagnostics:** Full language support for `.cgdsl` files with real-time error checking.
* **Automatic FSM Generation:** Automatically generates a **Finite State Machine (FSM)** of your game flow in `.dot` (Graphviz) format.
* **Visual Diagrams:** Renders your game logic into `.png` diagrams for easy documentation and debugging.
* **Data Serialization:** Compiles your game description into a structured `.json` file, ready to be imported into your game engine or backend.



## 🛠️ How to Use

1.  Create a new file with the `.cgdsl` extension.
2.  Define your game states, players, and card logic.
3.  The extension automatically maintains three outputs in your workspace:
    * `*.dot`: The raw graph description for Graphviz.
    * `*.png`: A high-resolution visual map of the game's Finite State Machine.
    * `*.json`: The serialized game data for programmatic use.

## 📋 Requirements

* **Graphviz:** To generate the `.png` and `.dot` files, ensure you have [Graphviz](https://graphviz.org/download/) installed and added to your system's PATH.
* **Platform Support:** Includes native binaries for Linux, Windows, and macOS.

## ⚙️ Extension Settings

This extension contributes the following settings:

* `cgdsl.outputDirectory`: Specify a custom subfolder where `.png` and `.json` files should be saved.
* `cgdsl.enableLivePreview`: Toggle whether images are re-generated automatically on every save.

## ⚠️ Known Issues

* Extremely large game files may take a moment to render complex `.png` diagrams.
* Ensure your workspace has write permissions for the output folder.

## 📝 Release Notes

### 1.0.0
* Initial release of CGDSL.
* Automatic export of PNG, DOT, and JSON.
* High-performance Language Server written in Rust.

---

**Happy Game Designing!**