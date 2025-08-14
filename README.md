# MCG

A Rust workspace for a browser-based card game. The frontend (client) compiles to WebAssembly (WASM) and renders with eframe/egui. The backend (server) is an Axum HTTP/WebSocket server that serves the SPA and provides a real-time poker demo. A shared crate contains the serialized message types and supporting structures.

## Quick start

- Prerequisites: Rust (stable toolchain), `wasm-pack` in PATH, and the `just` task runner.

- Build the WASM bundle (outputs to repo-root ./pkg):
  - `just build`              # release (optimized)
  - `just build dev`          # debug/dev
  - `just build profiling`    # profiling (same flags as debug currently)

- Run the backend server (serves /, /pkg, /media, and /ws):
  - `just server`             # default 1 bot
  - `just server 3`           # with 3 bots

- Build then run together:
  - `just start`              # release + server with 1 bot
  - `just start dev`          # dev + server with 1 bot
  - `just start release 3`    # release + server with 3 bots

Notes
- The server binds to the first available port starting at 3000 and logs the chosen URL (e.g., http://localhost:3000). Open that URL in the browser.
- The server assumes current working directory is the repo root to serve ./pkg and ./media.
- wasm-pack builds are run from client/ and emit to ../pkg (repo root). If a client/pkg directory exists, prefer the root pkg output.

## Headless CLI (for automation and testing)

A minimal CLI is provided to exercise the same WebSocket protocol as the GUI client. It is useful for smoke tests and AI agents.

- Run directly:
  - `cargo run -p mcg-server --bin mcg-cli -- [GLOBAL-OPTS] <COMMAND> [ARGS]`
- Or use the Just recipe (forwards all arguments after `--`):
  - `just cli -- [GLOBAL-OPTS] <COMMAND> [ARGS]`

Global options
- `--server` Base server URL (default: `http://localhost:3000`). Accepts http(s):// or ws(s)://; the CLI normalizes to ws(s) and forces path `/ws`.
- `--name`   Join name to use (default: `CLI`).
- `--wait-ms` How long to wait for state updates after a command (default: 1200ms). Useful to capture bot activity.
- `--json` Output JSON instead of a colorful, human-readable summary.

Commands (examples)
- Join and print first State:
  - `just cli join`
- Request latest State:
  - `just cli -- state`
- Send actions:
  - Fold: `just cli -- action fold`
  - Check/Call: `just cli -- action check-call`
  - Bet 20: `just cli -- action bet --amount 20`
- Advance hand:
  - `just cli -- next-hand`
- Reset game with bots:
  - `just cli -- reset --bots 3`

Output
- Default: a concise, colorized summary with stage, pot, players (including whose turn), board and your cards (if available), and a readable action log.
- With `--json`: pretty-printed `GameStatePublic` JSON.

## Workspace layout

- `client/`: WASM/egui frontend and all UI/game/screen code
- `server/`: Axum-based HTTP + WebSocket backend (serves SPA and game)
- `shared/`: Types shared between client and server (serde-serializable protocol and game data)
- `pkg/`: wasm-pack output (mcg.js, mcg_bg.wasm, mcg.d.ts) loaded by `index.html`
- `index.html`: loads `pkg/mcg.js` and starts the game on a full-screen canvas

## Adding new screens (frontend)

To add a new screen:
1) Add a `ScreenType` variant and its metadata in `client/src/game/screens/mod.rs`.
2) Implement a `ScreenWidget` for the screen and `ScreenFactory::create()`.
3) Register it in `ScreenType::create_screen` and wire it in the `App` screen match in `client/src/game.rs`.

Code examples

1. Declare and register your screen in `client/src/game/screens/mod.rs`:

```rust
// Add module and re-export
pub mod my_new_screen;
pub use my_new_screen::MyNewScreen;

// Add to the ScreenType enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScreenType {
    // ...
    MyNewScreen,
}

impl ScreenType {
    /// Add metadata for your screen
    pub fn metadata(&self) -> ScreenMetadata {
        match self {
            // ...
            ScreenType::MyNewScreen => ScreenMetadata {
                display_name: "My New Screen",
                icon: "🆕",
                url_path: "/my-new-screen",
                description: "Demo screen showing how to extend the UI",
                show_in_menu: true,
            },
        }
    }

    /// Make it show up in the main menu (optional)
    pub fn menu_screens() -> Vec<ScreenType> {
        vec![
            // ...
            ScreenType::MyNewScreen,
        ]
    }

    /// Map ScreenType -> concrete screen instance
    pub fn create_screen(self) -> Box<dyn ScreenWidget> {
        match self {
            // ...
            ScreenType::MyNewScreen => MyNewScreen::create(),
        }
    }
}
```

2. Implement the screen at `client/src/game/screens/my_new_screen.rs`:

```rust
use super::{AppInterface, ScreenFactory, ScreenWidget};
use eframe::Frame;

pub struct MyNewScreen {
    // your state here
}

impl MyNewScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl ScreenWidget for MyNewScreen {
    fn ui(&mut self, _app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        ui.heading("My New Screen");
        ui.label("Hello from a custom screen!");
    }
}

impl ScreenFactory for MyNewScreen {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(MyNewScreen::new())
    }
}
```

3. Wire the screen into the App in `client/src/game.rs`:

```rust
// 1) Add a field on App
pub struct App {
    // ...
    my_new_screen: screens::MyNewScreen,
    // ...
}

// 2) Initialize it in App::new()
Self {
    // ...
    my_new_screen: screens::MyNewScreen::new(),
    // ...
}

// 3) Route it in the update match
match self.current_screen {
    // ...
    ScreenType::MyNewScreen => self.my_new_screen.ui(&mut app_interface, ui, frame),
    // ...
}
```

Routing
- On wasm targets, a small `Router` syncs `ScreenType` with the browser URL using History/Location and popstate.

## Test in the browser

The easiest way is to run a dev build and start the server, then open the printed URL.

- One command (recommended):
  - `just start dev`
    - Builds the WASM bundle with wasm-pack into `./pkg/`
    - Starts the Axum server that serves `/`, `/pkg`, `/media`, and the WebSocket endpoint at `/ws`
    - Binds to the first available port starting at 3000 and prints the chosen URL
- Manual alternative:
  - `just build dev`
  - `just server` (or `just server 3` to run with 3 bots)

Then open the printed URL in your browser (e.g., http://localhost:3000).

In the app
- From the Main Menu, open “Poker Online”.
- Name: enter your display name (default is fine).
- Server: the server URL (default `http://localhost:3000`). Update it if the server chose a different port.
- Connect: creates/joins a single-player game (bots are added server-side). You’ll see bots act with short delays and the UI update live.
- Action row: click Fold / Check/Call / Bet to act; on Showdown, use “Next Hand”.
- Clipboard: use “Copy to clipboard” in the log to export a concise game summary.
- QR scan: click “Scan QR” beside the Server field to fill the server URL by scanning a QR code. Allow camera access when prompted. The popup is safe to close/reopen and shows a friendly message if no camera is available.

Hot reload loop
- After client changes: `just build dev` and refresh the browser tab.
- If the server is already running, it will serve the updated `./pkg` artifacts.

Troubleshooting
- Blank page or missing game:
  - Ensure `wasm-pack` is installed and in PATH.
  - Confirm `./pkg/mcg.js` exists and `index.html` loads it.
  - Check the browser console for WASM load errors.
- Can’t connect from the Poker Online screen:
  - Verify the server log shows the bound URL and that the Server field uses that exact URL.
  - The WebSocket endpoint is `/ws` under the same origin.
- Camera/QR issues:
  - Allow camera permission when asked. Some browsers restrict camera on non-secure origins; localhost is typically permitted.
- Port conflicts:
  - If 3000 is busy, the server picks the next free port (3001, …). Use the printed URL or update the Server field accordingly.

## Development workflow

1) `just start dev` to build and run
2) Open the printed URL (e.g., http://localhost:3000)
3) Iterate on client changes with `just build dev` and refresh the browser

## Testing and linting

- Tests (if present):
  - `cargo test --workspace`
  - `cargo test -p mcg-shared`
  - `cargo test -p mcg-server game::state::tests::your_test_name`
- Lint with Clippy (fail on warnings):
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Format:
  - `cargo fmt --all`

## License

Dual-licensed under MIT and Apache-2.0.
