---
outline: deep
---

# Frontend

**frontend** is the browser client of MCG.
It is written in Rust, compiled to WebAssembly (WASM), and renders the user
interface with `egui`/`eframe`.
The crate contains application orchestration, screen implementations, reusable
widgets, browser integration, and the WebSocket boundary to the local backend.

The crate is named `frontend`.
It is distinct from the Card Game Description Language compiler crate named
`front_end`.

## Module Layout

The source tree separates application-wide orchestration from screens and
reusable widgets:

```text
frontend/src/
├─ app.rs                  # FrontendApp, FrontendInterface, FrontendEvent
├─ app/
│  ├─ state.rs             # FrontendState
│  └─ websocket.rs         # WebSocketConnection and MessageSender
├─ screens.rs              # Screen module declarations and re-exports
├─ screens/                # Concrete screens and screen-local state
├─ widgets/
│  ├─ mod.rs               # Widget module declarations
│  ├─ screen.rs            # Screen traits, typed IDs, and registry
│  ├─ card.rs              # Card traits and implementations
│  ├─ field.rs             # Field traits and implementations
│  ├─ camera.rs            # Browser camera lifecycle and frame capture
│  ├─ qr_scanner.rs        # QR decoding UI
│  ├─ hardcoded_cards.rs   # Preconfigured decks
│  └─ theme.rs             # Theme constants and DPI helpers
├─ router.rs               # Browser URL routing
└─ lib.rs                  # WASM entry point
```

## Architecture & API

### Application Types

- **`FrontendApp`** @
  [frontend/src/app.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/app.rs)
  implements `eframe::App`.
  It owns the current screen, screen instances, global frontend state, router,
  WebSocket connection, and inbound event channels.
- **`FrontendInterface`** @
  [frontend/src/app.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/app.rs)
  is the capability boundary passed to screens.
  Its fields are private.
  Screens use methods such as `change_screen`, `send_msg`, `connect`,
  `create_game`, and `start_game` instead of accessing the owning `FrontendApp`
  or raw WebSocket.
- **`FrontendEvent`** @
  [frontend/src/app.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/app.rs)
  represents deferred application-level operations.
  It currently supports typed screen changes, starting the generic
  drag-and-drop game, and exiting a game.
- **`FrontendState`** @
  [frontend/src/app/state.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/app/state.rs)
  contains application-wide data:
  player name, server address, DPI settings, theme selection, and the screen
  registry.

### Screen Traits and Registry

- **`ScreenWidget`** @
  [frontend/src/widgets/screen.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/widgets/screen.rs)
  defines the runtime behavior of a screen:
  - `ui` renders the screen and handles user input.
  - `on_message` receives typed `Backend2FrontendMsg` values dispatched by
    `FrontendApp`.
  - `on_exit` releases screen-owned resources before navigation.
- **`ScreenDef`** defines compile-time metadata and a factory for a screen.
- **`ScreenId`** wraps Rust's `TypeId`. Navigation therefore uses screen types
  rather than path strings.
- **`ScreenRegistry`** maps `ScreenId`s and URL paths to metadata and factories.
  Screen instances are created lazily.
- **`impl_screen_def!`** generates a `ScreenDef` implementation for screen
  types that implement `Default`.

### Card and Field Traits

- **`CardEncoding`** @
  [frontend/src/widgets/card.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/widgets/card.rs)
  maps a card to the encoding required by mental-card-game operations and
  exposes masking/unmasking behavior.
- **`CardConfig`** @
  [frontend/src/widgets/card.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/widgets/card.rs)
  defines visual card rendering, cardinality, encoding width, and natural size.
- **`FieldWidget`** @
  [frontend/src/widgets/field.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/widgets/field.rs)
  defines a renderable card container.
- **`SimpleCard`**, **`DirectoryCardType`**, and **`SimpleField`** are the
  provided implementations.
  `SimpleField` supports stacked and horizontal layouts as well as selection
  and drag-and-drop.
- **`hardcoded_cards`** @
  [frontend/src/widgets/hardcoded_cards.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/widgets/hardcoded_cards.rs)
  provides preconfigured decks backed by media files served by the backend.

## Initialization & Lifecycle

The browser calls the `#[wasm_bindgen]`-exported `start` function in
`frontend/src/lib.rs`.
It installs image loaders, creates a `FrontendApp`, and starts the `eframe` web
runner:

```rust
#[wasm_bindgen]
pub fn start(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    let init = Box::new(|cc: &eframe::CreationContext| {
        install_image_loaders(&cc.egui_ctx);
        let app = FrontendApp::new(cc.egui_ctx.clone());
        Ok(Box::new(app) as Box<dyn eframe::App>)
    });
    start_game(canvas, init)
}
```

`FrontendApp` implements the `eframe::App` trait.
Its `update` method is part of  eframe's application lifecycle and is invoked
by the framework whenever a new UI frame must be produced.
It therefore serves as the frontend's main loop.

For each `FrontendApp::update` call, the application:

1. processes pending input and messages,
2. renders the current UI,
3. and applies queued application events.

## State Management

Only application-wide data belongs in `FrontendState`.
Screen-specific rendering data, edit buffers, connection feedback, and game
projections are owned by their corresponding `ScreenWidget`.

For example, `PokerOnlineScreen` owns its `Option<PokerStatePublic>` and
connection manager.
QR screens own their scanners and `Epoch` values.
These values are no longer kept in a global client-state monolith.

`FrontendInterface::state()` and `state_mut()` currently expose
`FrontendState`; further narrowing this API into purpose-specific accessors is
tracked by the
[Frontend State Decoupling](../project/milestones.md#frontend-state-decoupling)
milestone.

## Typed Navigation and Screen Events

Screens request navigation by target type:

```rust
impl FrontendInterface {
  pub fn change_screen<T: ScreenDef + 'static>(&mut self) { ... }
}
```

`FrontendInterface` converts the target type into a `ScreenId` and queues a
`FrontendEvent::ChangeScreen`.
`FrontendApp` validates it against the registry, calls `on_exit` on the old
screen, and updates the router with the registered path.

This keeps URL strings in `ScreenMetadata` and removes route "magic strings"
from screen logic.

## Networking

### Connection Ownership

A single **`WebSocketConnection`** @
[frontend/src/app/websocket.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/app/websocket.rs)
is owned by `FrontendApp`.
Screens never receive the raw `web_sys::WebSocket`.
They interact through `FrontendInterface` methods or the narrow `MessageSender`
trait.

Connecting installs application-level `onopen`, `onmessage`, `onerror`, and
`onclose` callbacks.

### Incoming Messages

The browser callback deserializes text frames into `Backend2FrontendMsg` and
sends them through a standard MPSC channel:

```text
web_sys::WebSocket::message
        │
        ▼
mpsc::Sender<Backend2FrontendMsg>
        │
        ▼
FrontendApp::dispatch_messages
        │
        ▼
active ScreenWidget::on_message
```

Screens neither register listener closures nor select an "active listener".
`FrontendApp` owns routing and dispatches each queued message to the active
screen.

Error and close events use separate MPSC channels and are logged by
`FrontendApp`.

### Sending Messages

Screens send typed protocol values through the interface:

```rust
impl FrontendInterface {
  pub fn send_msg(&mut self, msg: Frontend2BackendMsg) { ... }
}
```

Specialized operations such as `create_game` are also exposed.
The serialized WebSocket payload remains private to `WebSocketConnection`.

### Event-Driven Repainting

The application does not request a repaint unconditionally on every update.
Successful incoming WebSocket messages call `egui::Context::request_repaint`.
Time-dependent screens and widgets have to request their own repaint while
active;
for example, the QR scanner repaints while its camera popup is open and the QR
transmitter schedules its next frame.

## Drag & Drop

The generic game demo uses `egui` drag-and-drop while keeping widget rendering
separate from game-state mutation.

1. `SimpleField` in `frontend/src/widgets/field.rs` publishes a
   `DNDSelector::Index` payload and records the local card index.
2. `Game` in `frontend/src/screens/game.rs` translates the local index into a
   logical source or destination (`Player` or `Stack`).
3. Once both endpoints are present, `GameState::move_card` mutates the fields
   and the temporary drag/drop state is cleared.

```rust
impl ScreenWidget for Game<DirectoryCardType> {
  fn ui(&mut self, app_interface: &mut FrontendInterface, ui: &mut egui::Ui, frame: &mut Frame) {
    // ...
    
    if let (Some(source), Some(destination)) = (self.drag, self.drop) {
      self.game_state.move_card(source, destination);
      self.drag = None;
      self.drop = None;
    }
    // ...

  }
}
```

## Camera and QR Scanning

QR support is split into two reusable widgets:

- **`Camera`** @
  [frontend/src/widgets/camera.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/widgets/camera.rs)
  owns the `MediaStream`, video element, canvas, frame texture, and front/back
  camera selection.
  Starting and stopping are safe across asynchronous camera initialization.
- **`QrScanner`** @
  [frontend/src/widgets/qr_scanner.rs](https://github.com/mentalcardgames/mcg/blob/main/frontend/src/widgets/qr_scanner.rs)
  renders the scanner popup, captures every frame for preview, and attempts QR
  decoding every fifth frame.

`QrDecodeTarget` selects text or binary output:

```rust
use crate::widgets::qr_scanner::{QrDecodeTarget, QrScanner};

#[derive(Default)]
struct MyScreen {
    scanner: QrScanner,
    text: String,
    bytes: Vec<u8>,
}

impl ScreenWidget for MyScreen {
  fn ui(&mut self, app_interface: &mut FrontendInterface, ui: &mut egui::Ui, frame: &mut Frame) {
    // Text mode closes after the first successful decode.
    self.scanner.button_and_popup(
      ui,
      ui.ctx(),
      QrDecodeTarget::String(&mut self.text),
    );

    // Binary mode remains open so a stream of QR frames can be collected.
    self.scanner.button_and_popup(
      ui,
      ui.ctx(),
      QrDecodeTarget::Binary(&mut self.bytes),
    );
  }
}
```

## Extensibility

### How to Add a Screen

Create the screen under `frontend/src/screens/`, implement `ScreenWidget`, and
define its metadata.
The macro is the shortest option for default-constructible screens:

```rust
use egui::Ui;
use eframe::Frame;

#[derive(Default)]
pub struct MyScreen;

impl ScreenWidget for MyScreen {
    fn ui(&mut self, interface: &mut FrontendInterface, ui: &mut Ui, frame: &mut Frame) {
        ui.label("Hello from MyScreen!");
        if ui.button("Back").clicked() {
            interface.change_screen::<crate::screens::MainMenu>();
        }
    }
}

crate::impl_screen_def!(
    MyScreen,
    "/myscreen",
    "My Screen",
    "🌟",
    "A custom example screen",
    true
);
```

Furthermore, new screens should be exported in `frontend/src/screens.rs` and
have to be registered:

```rust
impl ScreenRegistry {
  pub fn new() -> Self {
    let mut reg = Self { .. };
    
    // Register all screens by calling their ScreenDef implementations
    reg.register::<MainMenu>();
    reg.register::<MyScreen>();
    // ...
    reg
  }
}
```

Implement `ScreenWidget::on_message` when the screen wants to consume backend
messages and `on_exit` when it owns resources that must be released during
navigation.

### How to Add a Card Type

Implement `CardEncoding` for internal representation and `CardConfig` for their
visual representation:

```rust
struct MyCard {
    id: usize,
    masked: bool,
}

impl CardEncoding for MyCard {
    // Implement `t`, `is_masked`, `mask`, and `open`.
}

struct MyCardVisuals;

impl CardConfig for MyCardVisuals {
    // Implement `img`, `T`, `w`, and `natural_size`.
}
```

### How to Add a Field Layout

`SimpleField` provides horizontal and stacked layouts.
A new layout can be implemented as another `FieldWidget`:

```rust
struct VerticalField {
    // cards and layout configuration
}

impl FieldWidget for VerticalField {
    fn draw(&self) -> impl egui::Widget {
        move |ui: &mut egui::Ui| {
            // Draw the field and return its response.
            ui.response()
        }
    }
}
```
