# Data Structures & Protocol

This document details the core data structures and the communication protocol used in the MCG project.

## Core Domain Objects (`shared::game`)

TODO: The struct `GameStatePublic` is not inside `shared::game`, it is inside `shared::messages`.

### `GameStatePublic`
The primary structure sent to clients. It contains everything a client is allowed to know.

```rust
pub struct GameStatePublic {
    pub players: Vec<PlayerPublic>, // Public info about each player (cards are masked if not self)
    pub community: Vec<Card>,       // Board cards
    pub pot: u32,
    pub to_act: PlayerId,           // Whose turn is it?
    pub stage: Stage,               // PreFlop, Flop, Turn, River, Showdown
    pub action_log: Vec<ActionEvent>, // History of what happened
    // ... betting info (current_bet, min_raise)
}
```

TODO: The struct `PlayerPublic` is not inside `shared::game`, it is inside `shared::player`.

### `PlayerPublic`
Public view of a player.
```rust
pub struct PlayerPublic {
    pub id: PlayerId,
    pub name: String,
    pub stack: u32,
    pub bet: u32,       // Current bet in this street
    pub is_active: bool,// Has not folded
    pub cards: Option<[Card; 2]>, // Only present for "self" or at Showdown
}
```

## Communication Protocol (`shared::messages`)

Communication happens via WebSockets using JSON serialization (`serde_json`).

### Client -> Server (`ClientMsg`)
| Variant | Data | Description |
|:---|:---|:---|
| `Action` | `{ player_id, action }` | Perform a game action (Fold, Check, Call, Bet). |
| `NewGame` | `{ players }` | Reset the lobby and start a new game with given config. |
| `NextHand` | `null` | Advance to the next hand after a showdown. |
| `RequestState` | `null` | Ask server to resend the latest `State`. |
| `QrReq` | `filename` | **(Dev/Test)** Request a test file content for QR generation. |

### Server -> Client (`ServerMsg`)
| Variant | Data | Description |
|:---|:---|:---|
| `State` | `GameStatePublic` | The new authoritative game state. Sent after any change. |
| `Error` | `String` | Error message (e.g., "Not your turn"). |
| `QrRes` | `Box<[u8]>` | **(Dev/Test)** Binary content of the requested test file. |

## QR Protocol Data Structures (`crates/qr_comm`)

The QR communication uses a custom framing protocol to support "fountain coding".

### `HEADER` (39 bytes)
Every QR code (Frame) starts with a header.
-   **Epoch ID**: Identifies the data stream session.
-   **Participant ID**: Identifies the sender (0-15).
-   **Sequence Info**: Helps ordered decoding.

### `Frame`
A single unit of transmission (one QR code).
-   Contains coding factors (coefficients) and the payload fragment.
-   Size: ~858 bytes (configurable via constants).

### `Epoch`
Represents the collection of frames for a specific transmission session.
-   Accumulates received frames.
-   Performs Gaussian elimination (or similar matrix solving) to solve for the original fragments once enough linearly independent frames are received.
