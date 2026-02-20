# mcg_visual

**mcg_visual** is a Rust-based WebAssembly (WASM) framework designed for visualizing card games. It leverages `egui` for the user interface, providing a flexible and responsive environment for card game development and prototyping.

## Architecture & API

The core interaction logic is built around a set of traits and structs that define how screens, cards, and fields behave.

### Key Traits

-   **`ScreenWidget`** @ [frontend/src/game/screens/mod.rs](../frontend/src/game/screens/mod.rs):
    -   **Purpose**: Defines the **rendering logic** and behavior of a screen.
    -   **Usage**: Implement this trait to create new views (e.g., Main Menu, Game Setup). The `ui` method, called every frame, handles both logic updates and UI drawing.
    -   **Navigation**: Screens can request transitions to other screens via the `AppInterface` (passed as a parameter to `ui`). See [AppInterface](../frontend/src/game/screens/mod.rs).

-   **`ScreenDef`** @ [frontend/src/game/screens/mod.rs](../frontend/src/game/screens/mod.rs):
    -   **Purpose**: Defines the **metadata** and factory for a screen.
    -   **Usage**: Implement this to provide static information (path, display name, icon) and a constructor function. This allows the `ScreenRegistry` to list and instantiate screens dynamically.

-   **`CardEncoding`** @ [frontend/src/game/card.rs](../frontend/src/game/card.rs):
    -   **Purpose**: Acts as an interface to make custom types accessible as cards for mental card games in academia.
    -   **Usage**: Implement this in order to translate specific cards (e.g., Suit/Rank, ID) into an encoding used by mental card games. It provides operations like to both check whether a card is masked (face down) or open (face up) and to mask or unmask it.

-   **`CardConfig`** @ [frontend/src/game/card.rs](../frontend/src/game/card.rs):
    -   **Purpose**: Defines the visual representation of a card.
    -   **Usage**: Implement this to tell the system how to render a card specifically. It maps the logical `CardEncoding` to an `egui::Image`.

-   **`FieldWidget`** @ [frontend/src/game/field.rs](../frontend/src/game/field.rs):
    -   **Purpose**: Defines a container that holds cards.
    -   **Usage**: Used to render areas where cards exist, such as a draw pile, discard pile, or a player's hand.

### Core Structs & Modules

-   **`App`** @ [frontend/src/game.rs](../frontend/src/game.rs):
    -   The main entry point that manages the registration and switching of `ScreenWidget`s via the `ScreenRegistry`.
    -   **Entry Point**: The `update` method (from the `eframe::App` trait) is the main loop where the application state is updated and the UI is rendered.

- **`ClientState`** @ [frontend/src/store.rs](../frontend/src/store.rs):
    - Holds the important state information of the client e.g. backend address, network messages, etc.

-   **`SimpleField`** @ [frontend/src/game/field.rs](../frontend/src/game/field.rs):
    -   A full implementation of `FieldWidget`.
    -   **Purpose**: Serves as a default container for card storage.
    -   **Features**: Supports `Stack` (cards on top of each other) and `Horizontal` (cards side-by-side) layouts. Handles the drag-and-drop logic for cards within or between fields.

-   **`SimpleCard`** @ [frontend/src/game/card.rs](../frontend/src/game/card.rs):
    -   A implementation of `CardEncoding` that enumerate cards by a number and support masking and unmasking operations.

-   **`DirectoryCardType`** @ [frontend/src/game/card.rs](../frontend/src/game/card.rs):
    -   A full implementation of `CardConfig` used for `SimpleCard`.
    -   **Features**: Configuring a deck where card images are loaded from the backend at a specific directory.

-   **`hardcoded_cards`** (Module) @ [frontend/src/hardcoded_cards.rs](../frontend/src/hardcoded_cards.rs):
    -   **Purpose**: Provides factory functions to create pre-configured card decks (e.g., standard 52-card deck). This is not a struct but a helper module to easily instantiate `DirectoryCardType` with standard assets.

### Initialization & Lifecycle

The frontend application starts via the `start` function in `frontend/src/lib.rs`, which is marked with `#[wasm_bindgen]`.

> **Note**: The `#[wasm_bindgen]` attribute is critical here. It exposes this Rust function to the JavaScript environment, allowing the browser's JS code to call `frontend.start()` to launch the WebAssembly application.

```rust
// frontend/src/lib.rs
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    // ... setup and start eframe ...
}
```

Once started, the application enters its main loop. The `App` struct (in `frontend/src/game.rs`) implements `eframe::App`, and its `update` method is called every frame by the browser/renderer.

-   **`App::new`**: Initializes the global state (`ClientState`), registers screens, and sets up the router.
-   **`App::update`**: 
    1.  Processes pending messages (from WebSocket/Network). This is currently omitted.
    2.  Handles URL changes (routing).
    3.  Renders the top navigation bar.
    4.  Delegates rendering to the active `ScreenWidget` based on the current path.

### Event Handling

The `AppInterface` struct is passed to every screen's `ui` method. It holds a mutable reference to the `AppEvent` queue. Screens push events (like `ChangeRoute` or `StartGame`) to this queue. After the screen's `ui` method returns, `App::update` drains this queue and executes the events. This pattern avoids borrow checker conflicts where a screen tries to mutate the `App` that owns it.

### State Management

Global state is held in `ClientState` (in `frontend/src/store.rs`). It contains data shared across the application, such as the current game state, connection status, and player settings. It is accessible via `AppInterface.state()` in any screen.

Local state (like specific UI toggles or temporary input buffers) should remain inside the specific `ScreenWidget` struct.

### Networking (Backend Connection)

Communication with the backend is handled via WebSockets using the `WebSocketConnection` struct (in `frontend/src/game/websocket.rs`).

#### Connecting

You connect by providing callbacks for messages, errors, and disconnection. This is typically done within a screen or a connection manager.

```rust
let mut ws = WebSocketConnection::new();
ws.connect(
    "127.0.0.1:3000",
    players_config,
    move |msg: ServerMsg| {
        // Handle incoming message (e.g. queue it to ClientState)
    },
    move |err| { log(err); },
    move |reason| { log(reason); }
);
```

#### Receiving Messages

Currently you need to have the `WebSocketConnection` as a field in your screen.

1.  **Callback**: The WebSocket entry receives a message.
2.  **Queueing**: The message is pushed to a thread-safe queue.
3.  **Processing**: The main `App::update` or the screen's `ui` method pops messages from the queue and applies them to the state.

#### Sending Messages

Sending is straightforward using the `WebSocketConnection::send_msg` method, which serializes `ClientMsg` to JSON.

```rust
ws.send_msg(&ClientMsg::Action { ... });
```

### Drag & Drop (DnD)

The system leverages `egui`'s native drag and drop capabilities to allow intuitive card interaction. The architecture separates the **Visual Widget** (the Field) from the **Game Logic** (the State).

1.  **The Payload (`DNDSelector`)**:
    We define a specific payload type that carries information about what is being dragged.
    ```rust
    // frontend/src/game/screens/game.rs
    pub enum DNDSelector {
        Player(usize, usize), // (Player Index, Card Index)
        Stack,                // From the top of the stack
        Index(usize),         // Generic index
    }
    ```

2.  **The Source (SimpleField)**:
    When drawing the field, if a user starts dragging a card, the field acts as the source and sets the payload.
    ```rust
    // frontend/src/game/field.rs @ SimpleField::draw_horizontal
    if ui.response().drag_started() {
        // Set the payload to indicate WHICH card is being dragged
        ui.response().dnd_set_drag_payload(DNDSelector::Index(idx));
    }
    ```

3.  **The Detection (Game Loop)**:
    In your main game loop (`ScreenWidget::ui`), you check if a payload was released over a specific area (the drop target).
    ```rust
    // frontend/src/game/screens/game.rs @ impl ScreenWidget::ui
    // Draw the stack (the drop target)
    let response = ui.add(stack.draw());

    // Check if something valid was dropped onto the stack
    if let Some(payload) = response.dnd_release_payload::<DNDSelector>() {
        // 'self.drop' stores WHERE we dropped it
        self.drop = Some(DNDSelector::Stack);
    }
    ```

4.  **The Mutation**:
    Finally, you resolve the move by modifying the game state. This usually happens at the end of the update loop.
    ```rust
    // frontend/src/game/screens/game.rs @ impl ScreenWidget::ui
    if let (Some(source), Some(destination)) = (self.drag, self.drop) {
        // Move the card data from source field to destination field
        game_state.move_card(source, destination);
        
        // Reset state
        self.drag = None;
        self.drop = None;
    }
    ```

This separation allows for validation logic (e.g., checking if a move is legal before applying it) to be inserted easily in the `The Mutation` step.

### QR Code Scanning

The project uses the `QrScannerPopup` struct (in `frontend/src/qr_scanner.rs`) to handle camera input and QR detection directly in the browser.

-   **Usage**: The `QrScannerPopup` manages the camera and updates a target string buffer with the result.
-   **Integration**:
    ```rust
    // In your screen struct
    struct MyScreen {
        scanner: QrScannerPopup,
        result: String,
        raw_result: Vec<u8>
    }
    
    // In your ui() method
    self.scanner.button_and_popup(ui, ctx, &mut self.result, &mut self.raw_result);
    ```
    This single call renders the "Scan QR" button and handles the popup overlay, camera permissions, and decoding logic.

## Extensibility

### How to add a custom screen?

Screens are distinct views (e.g., Main Menu, Poker Table, QR Scanner). To add a new screen, you need to implement both the `ScreenWidget` (logic/view) and `ScreenDef` (metadata/registration) traits.

1.  **Create the Screen Struct**: Implement `ScreenWidget` (for rendering) and `ScreenDef` (for registration). Alternatively you can use the `impl_screen_def!` macro to generate the `ScreenDef` implementation.
    ```rust
    // frontend/src/game/screens/my_screen.rs
    pub struct MyScreen;
    
    impl ScreenWidget for MyScreen {
        fn ui(&mut self, app: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
            ui.label("Hello from MyScreen!");
            if ui.button("Back").clicked() {
                app.queue_event(AppEvent::ChangeRoute("/".into()));
            }
        }
    }
    
    impl ScreenDef for MyScreen {
        fn metadata() -> ScreenMetadata {
            ScreenMetadata {
                path: "/myscreen",
                display_name: "My Screen",
                icon: "🌟",
                description: "A custom example screen",
                show_in_menu: true,
            }
        }
        fn create() -> Box<dyn ScreenWidget> { Box::new(MyScreen) }
    }
    ```

2.  **Register the Screen**: Add it to `ScreenRegistry::new` in `frontend/src/game/screens/mod.rs`.
    ```rust
    // frontend/src/game/screens/mod.rs
    pub fn new() -> Self {
        // ...
        reg.register::<MyScreen>();
        // ...
    }
    ```

### How to add a custom card type?

The trait `CardEncoding` is used as an interface to provide data that can be used in a mental card game setting.
The trait `CardConfig` is used for visual rendering. The key point is to return `egui::Image` in the `CardConfig::img` method so the systems knows which image to display as your card.

```rust
// frontend/src/game/card/my_card.rs
struct MyCard { id: usize }
impl CardEncoding for MyCard { ... }

struct MyCardVisuals;
impl CardConfig for MyCardVisuals {
    fn img(&self, card: &impl CardEncoding) -> Image {
        // Return appropriate image based on card state
    }
    // ... define dimensions
}
```

### How to add custom Field Layouts?

With `SimpleField` you can create horizontal or stacked layouts. For more complex layouts, for a game like **Solitaire** where you need vertical columns of cards, you need to create a new struct that implements `FieldWidget`.

```rust
// frontend/src/game/field/vertical_field.rs
struct VerticalField { ... }
impl FieldWidget for VerticalField {
    fn draw(&self) -> impl egui::Widget {
        // Implementation for drawing cards vertically
    }
}
```
