# mcg_visual

**mcg_visual** is a Rust-based WebAssembly (WASM) framework designed for visualizing card games. It leverages `egui` for the user interface, providing a flexible and responsive environment for card game development and prototyping.

## Project Overview

This project provides a robust foundation for building card games with the following core features:

-   **WASM & Rust Powers**: High-performance game logic running in the browser.
-   **Immediate Mode UI**: Built on `egui` for rapid iteration and dynamic interfaces.
-   **Visual Card Management**:
    -   **Drag & Drop**: Intuitive mechanics for moving cards between different zones (stacks, player hands, etc.).
    -   **Local Asset Loading**: Capability to validly load card images from a local directory served via HTTP.
-   **Configurable Game State**: Support for configurable number of players and flexible initial game setups.
-   **Extensible Architecture**: Traits and structs designed for easy extension to support various card game rules and visuals.

## Architecture & API

The core interaction logic is built around a set of traits and structs that define how screens, cards, and fields behave.

### Key Traits

-   **`ScreenWidget`** @ [frontend/src/game/screens/mod.rs](../frontend/src/game/screens/mod.rs):
    -   **Purpose**: Defines a distinct screen or state in the application (e.g., Main Menu, Game Setup, Active Game).
    -   **Usage**: Implement this trait to create new views. The `ui` method handles the logic and UI rendering for that screen.
    -   **Navigation**: Screens can request transitions to other screens via the `AppInterface` (passed as a parameter to `ui`). See [AppInterface](../frontend/src/game/screens/mod.rs).

-   **`CardEncoding`** @ [frontend/src/game/card.rs](../frontend/src/game/card.rs):
    -   **Purpose**: Acts as an interface to make custom types accessible as cards for mental card games in academia.
    -   **Usage**: Implement this to define how your cards are represented in data (e.g., Suit/Rank, ID). It handles state like whether a card is masked (face down) or open (face up).

-   **`CardConfig`** @ [frontend/src/game/card.rs](../frontend/src/game/card.rs):
    -   **Purpose**: Defines the visual representation of a card.
    -   **Usage**: Implement this to tell the system how to render a card specifically. It maps the logical `CardEncoding` to an `egui::Image`.

-   **`FieldWidget`** @ [frontend/src/game/field.rs](../frontend/src/game/field.rs):
    -   **Purpose**: Defines a container that holds cards.
    -   **Usage**: Used to render areas where cards exist, such as a draw pile, discard pile, or a player's hand.

### Core Structs

-   **`App`** @ [frontend/src/game.rs](../frontend/src/game.rs):
    -   The main entry point that manages the registration and switching of `ScreenWidget`s via the `ScreenRegistry`.
    -   **Entry Point**: The `update` method (from the `eframe::App` trait) is the main loop where the application state is updated and the UI is rendered.

-   **`SimpleField`** @ [frontend/src/game/field.rs](../frontend/src/game/field.rs):
    -   A full implementation of `FieldWidget`.
    -   **Purpose**: Serves as a default container for card storage.
    -   **Features**: Supports `Stack` (cards on top of each other) and `Horizontal` (cards side-by-side) layouts. Handles the drag-and-drop logic for cards within or between fields.

-   **`DirectoryCardType`** @ [frontend/src/game/card.rs](../frontend/src/game/card.rs):
    -   A full implementation of `CardConfig`.
    -   **Purpose**: An example implementation for default 52 playing card sets that is able to display different images depending on the style that is chosen. 
    -   **Features**: configuring a deck where card images are loaded from a specific directory. Useful for quick prototyping with custom assets.

-   **`SimpleCard`** @ [frontend/src/game/card.rs](../frontend/src/game/card.rs):
    -   A full implementation of `CardEncoding`.
    -   **Purpose**: Represents a card in the default 52 playing card set.

## Extensibility

`mcg_visual` is designed to be extended. Here is how you can build upon it:

### 1. Custom Game Screens
Create a struct and implement `ScreenWidget`. Include this file in the module tree (e.g., in `frontend/src/game/screens/my_victory_screen.rs`).

```rust
// frontend/src/game/screens/my_victory_screen.rs
struct MyVictoryScreen;
impl ScreenWidget for MyVictoryScreen {
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
         egui::CentralPanel::default().show(ctx, |ui| {
             ui.heading("You Won!");
             if ui.button("Back to Menu").clicked() {
                 app_interface.queue_event(crate::game::AppEvent::ChangeRoute("/".to_string()));
             }
         });
    }
}
```

**Registration:**
Register the screen in `frontend/src/game/screens/mod.rs` inside `ScreenRegistry::new()`:

```rust
// frontend/src/game/screens/mod.rs
impl ScreenRegistry {
    pub fn new() -> Self {
        // ...
        reg.register::<MyVictoryScreen>(); // Register your new screen here
        // ...
    }
}
```

### 2. Custom Card Types
Implement `CardEncoding` for your card logic and `CardConfig` for visual rendering. The key point is to return `egui::Image` in the `CardConfig::img` method so the systems knows which image to display as your card.

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

### 3. Complex Layouts using `SimpleField`
You can compose multiple `SimpleField` instances to create complex board states. However, `SimpleField` is primarily designed for horizontal or stacked layouts.

**Example**: For a game like **Solitaire** where you need vertical columns of cards, you would need to create a new struct that extends `FieldWidget`. `SimpleField` is not capable of vertical card placement out of the box.

```rust
// frontend/src/game/field/vertical_field.rs
struct VerticalField { ... }
impl FieldWidget for VerticalField {
    fn draw(&self) -> impl egui::Widget {
        // Implementation for drawing cards vertically
    }
}
```

### 4. Custom Fields
If `SimpleField` doesn't meet your needs (e.g., you need a circular or grid layout), you can implement your own struct that accesses card data and implements `FieldWidget`.

## Features Detailed

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
    When drawing the field, if a user starts dragging a card, the field sets the payload.
    ```rust
    // frontend/src/game/field.rs @ SimpleField::draw_horizontal
    if ui.response().drag_started() {
        // Set the payload to indicate WHICH card is being dragged
        ui.response().dnd_set_drag_payload(DNDSelector::Index(idx));
    }
    ```

3.  **The Detection (Game Loop)**:
    In your main game loop (`ScreenWidget::ui`), you check if a payload was released over a specific area.
    ```rust
    // frontend/src/game/screens/game.rs @ impl ScreenWidget::ui
    // Draw the stack
    let response = ui.add(stack.draw());

    // Check if something valid was dropped onto the stack
    if let Some(payload) = response.dnd_release_payload::<DNDSelector>() {
        // 'self.drop' stores WHERE we dropped it
        self.drop = Some(DNDSelector::Stack);
    }
    ```

4.  **The Mutation**:
    Finally, you resolve the move by modifying the game state.
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
