# MCG

A Rust workspace for a browser-based card game. The frontend (crate: `frontend`) compiles
to WebAssembly (WASM) and renders with eframe/egui. The native node (crate: `native_mcg`) provides
an HTTP/WebSocket backend that serves the SPA and provides the real-time poker demo. A shared
crate contains the serialized message types and supporting structures.

## Quick start

- Prerequisites: Rust (stable toolchain), `wasm-pack` in PATH, and the `just` task runner.

- Build the WASM bundle (outputs to repo-root ./pkg):
  - `just build`              # release (optimized)
  - `just build dev`          # debug/dev
  - `just build profiling`    # profiling (same flags as debug currently)

- Run the native backend (serves /, /pkg, /media, and /ws):
  - `just backend`            # runs backend (bots configured via config file)

- Build then run together:
  - `just start`              # release build + backend

Notes
- The server binds to the first available port starting at 3000 and logs the chosen URL (e.g., http://localhost:3000). Open that URL in the browser.
- The native node assumes current working directory is the repo root to serve ./pkg and ./media.
- wasm-pack builds are run from the `frontend/` crate and emit to ../pkg (repo root). If a `frontend/pkg` directory exists, prefer the root `pkg` output.

## Headless CLI (for automation and testing)

A minimal CLI is provided to exercise the same WebSocket protocol as the GUI client. It is useful for smoke tests and AI agents.

- Run directly:
  - `cargo run -p native_mcg --bin mcg-cli -- [GLOBAL-OPTS] <COMMAND> [ARGS]`
- Or use the Just recipe (forwards all arguments after `--`):
  - `just cli -- [GLOBAL-OPTS] <COMMAND> [ARGS]`

Global options
- `--server` Base server URL (default: `http://localhost:3000`). Accepts http(s):// or ws(s)://; the CLI normalizes to ws(s) and forces path `/ws`.
- `--name`   Join name to use (default: `CLI`).
- `--wait-ms` How long to wait for state updates after a command (default: 1200ms). Useful to capture bot activity.
- `--json` Output JSON instead of a colorful, human-readable summary.

Commands (examples)
- Join and print first State:
  - `just cli join` (the CLI will connect, wait for `ServerMsg::Welcome` and the initial `State`, then it may send follow-up `ClientMsg` commands)
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

- `frontend/`: WASM/egui frontend and all UI/game/screen code (previously `client/`)
- `native_mcg/`: Native node containing the backend (HTTP + WebSocket + iroh), CLI, and native-only helpers (previously `server/`)
- `shared/`: Types shared between frontend and native_mcg (serde-serializable protocol and game data)
- `pkg/`: wasm-pack output (mcg.js, mcg_bg.wasm, mcg.d.ts) loaded by `index.html`
- `index.html`: loads `pkg/mcg.js` and starts the game on a full-screen canvas

## Adding new screens (frontend)

The client uses a small screen registry and two traits to separate compile-time metadata from runtime UI state: ScreenDef (provides metadata() and create()) and ScreenWidget (the object-safe runtime UI trait: ui()). To add a new screen follow these steps:

1) Create the screen module and type
- Add a new file under `frontend/src/game/screens/`, e.g. `my_new_screen.rs`.
- Implement a struct to hold the screen's runtime state and implement the ScreenWidget trait for it.
- Implement the ScreenDef trait for the type to provide ScreenMetadata and a factory (create()).

Example (frontend/src/game/screens/my_new_screen.rs):

```rust
use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
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

impl ScreenDef for MyNewScreen {
    fn metadata() -> ScreenMetadata {
        ScreenMetadata {
            path: "/my-new-screen",
            display_name: "My New Screen",
            icon: "🆕",
            description: "Demo screen showing how to extend the UI",
            show_in_menu: true,
        }
    }

    fn create() -> Box<dyn ScreenWidget> {
        Box::new(Self::new())
    }
}
```

2) Register and re-export the screen
- Add the module declaration and a public re-export in `client/src/game/screens/mod.rs` so the registry and other code can find it.
- Register the screen in the ScreenRegistry by adding a RegisteredScreen entry to the `regs` slice in `ScreenRegistry::new()` (see the existing pattern in that file).

Example edits to `frontend/src/game/screens/mod.rs`:

```rust
// at the top, add:
pub mod my_new_screen;
pub use my_new_screen::MyNewScreen;

// inside the `regs` array in ScreenRegistry::new(), add another entry:
RegisteredScreen {
    meta: MyNewScreen::metadata(),
    factory: MyNewScreen::create,
},
```

3) Special-case typed screens (optional)
- Some screens in this codebase are stored directly on App as typed fields (for example the main Game screen) instead of being created from the registry. If your screen needs to be owned by App as a typed instance (for faster access or special lifetime reasons), add a field on App, initialize it in App::new(), and render it as a special-case in the CentralPanel before the registry lookup.

Example (frontend/src/game.rs):

```rust
// add a field on App
pub struct App {
    // ...
    my_new_screen: screens::MyNewScreen,
    // ...
}

// initialize in App::new()
Self {
    // ...
    my_new_screen: screens::MyNewScreen::new(),
    // ...
}

// render special-case in the CentralPanel before the registry path handling:
if self.current_screen_path == "/my-new-screen" {
    self.my_new_screen.ui(&mut app_interface, ui, frame);
} else {
    // existing registry-based creation and rendering
}
```

4) Triggering navigation
- To navigate to your screen from code or UI, queue App events with AppEvent::ChangeRoute("/my-new-screen".to_string()) or call the Router on wasm targets.
- If you want the screen to appear in the main menu, set show_in_menu: true in its metadata and register it in the registry.

Notes and tips
- The ScreenRegistry is used to present menu entries and lazily create runtime screen instances for routes handled by path. Registered screens must provide unique URL-safe path values.
- Prefer implementing a ScreenDef + ScreenWidget and registering it in the registry for regular screens. Use typed App-owned screens only when necessary.
- After adding a screen, run `just build dev` and open the app (`just start dev`) to verify it appears in the menu and renders as expected.

Routing
- On wasm targets, a small `Router` syncs `ScreenType` with the browser URL using History/Location and popstate.

## Test in the browser

The easiest way is to run a dev build and start the server, then open the printed URL.

- One command (recommended):
  - `just start dev`
    - Builds the WASM bundle with wasm-pack into `./pkg/`
    - Starts the native backend that serves `/`, `/pkg`, `/media`, and the WebSocket endpoint at `/ws`
    - Binds to the first available port starting at 3000 and prints the chosen URL
- Manual alternative:
  - `just build dev`
  - `just backend`

Then open the printed URL in your browser (e.g., http://localhost:3000).

Hot reload loop
- After frontend changes: `just build dev` and refresh the browser tab.
- If the native backend is already running, it will serve the updated `./pkg` artifacts.

Configuration
- Bots are configured via the `mcg-server.toml` config file in the current directory
- The config file is created automatically on first run with default values (1 bot)
- Edit the config file to change the number of bots or other settings

Troubleshooting
- Blank page or missing game:
  - Ensure `wasm-pack` is installed and in PATH.
  - Confirm `./pkg/mcg.js` exists and `index.html` loads it.
  - Check the browser console for WASM load errors.
- Can't connect from the Poker Online screen:
  - Verify the backend log shows the bound URL and that the Server field uses that exact URL.
  - The WebSocket endpoint is `/ws` under the same origin.
- Camera/QR issues:
  - Allow camera permission when asked. Some browsers restrict camera on non-secure origins; localhost is typically permitted.
- Port conflicts:
  - If 3000 is busy, the server picks the next free port (3001, …). Use the printed URL or update the Server field accordingly.

## Development workflow

1) `just start dev` to build and run
2) Open the printed URL (e.g., http://localhost:3000)
3) Iterate on frontend changes with `just build dev` and refresh the browser

## Testing and linting

- Tests (if present):
  - `cargo test --workspace`
  - `cargo test -p shared`
  - `cargo test -p native_mcg game::state::tests::your_test_name`
- Lint with Clippy (fail on warnings):
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Format:
  - `cargo fmt --all`

## License

Dual-licensed under MIT and Apache-2.0.
