# MCG Project Refactoring Plan

## Executive Summary

This document outlines a comprehensive refactoring plan for the MCG (Mental Card Game) Rust project. The analysis revealed several organizational issues including oversized files, misplaced components, and unclear module boundaries. The plan addresses these issues through strategic decomposition, better module organization, and improved naming conventions.

## Identified Issues

### Oversized Files (Priority: High)
1. **`frontend/src/game/screens/poker_online.rs`** (1,179 lines)
   - Contains poker game logic, UI rendering, player management, and server communication
   - Should be split into multiple focused components

2. **`native_mcg/src/backend/state.rs`** (491 lines)
   - Contains server state management, lobby handling, and bot management logic
   - Mixes low-level state with high-level business logic

3. **`native_mcg/src/eval.rs`** (443 lines)
   - Contains poker hand evaluation, card manipulation utilities, and constant definitions
   - Should separate card representation from evaluation logic


### Misplaced Components (Priority: Medium)

#### Frontend Issues:
1. **`frontend/src/game.rs`** (319 lines)
   - Contains App struct, router integration, and screen management
   - Should separate application framework from game-specific logic

#### Native Backend Issues:
1. **`native_mcg/src/pretty.rs`** (317 lines)
   - Contains output formatting, display utilities, and CLI helpers
   - Should be split by output target (console vs. structured)

2. **`native_mcg/src/game/engine.rs`** (286 lines)
   - Contains game loop, rule enforcement, and state transitions
   - Core engine logic should be separated from rule implementation

3. **`native_mcg/src/backend/iroh.rs`** (297 lines)
   - Contains P2P networking, connection management, and protocol handling
   - Should separate transport layer from protocol logic

### Module Organization Issues (Priority: Medium)

#### Frontend Structure:
- `frontend/src/game/` mixes UI screens with game logic
- Card-related functionality scattered across multiple files
- No clear separation between UI components and game mechanics

#### Native Backend Structure:
- `native_mcg/src/game/` mixes engine logic with poker-specific rules
- Backend functionality spread across multiple modules without clear boundaries
- CLI tools mixed with core server functionality

## Refactoring Plan

### Phase 1: Break Down Oversized Files

#### Frontend: poker_online.rs Refactoring
**Target:** Split 1,179-line file into focused modules

1. **Create `frontend/src/game/screens/poker/` directory:**
   ```
   poker/
   ├── mod.rs
   ├── screen.rs         # PokerOnlineScreen struct (main UI)
   ├── player_manager.rs # Player configuration and management
   ├── ui_components.rs  # Reusable UI components for poker
   ├── game_controller.rs # Poker game state management
   └── connection.rs     # WebSocket and server communication
   ```

2. **Extract concerns:**
   - Player management logic → `player_manager.rs`
   - UI components (tables, cards, buttons) → `ui_components.rs`
   - Game state tracking → `game_controller.rs`
   - Server communication → `connection.rs`
   - Main screen structure → `screen.rs`

#### Native Backend: state.rs Refactoring
**Target:** Split 491-line file into domain-specific modules

1. **Create `native_mcg/src/server/` directory:**
   ```
   server/
   ├── mod.rs
   ├── state.rs         # Core AppState and shared state
   ├── lobby.rs         # Game lobby management
   ├── session.rs       # Client session handling
   └── bot_manager.rs    # Bot lifecycle management
   ```

2. **Extract concerns:**
   - Lobby and game management → `lobby.rs`
   - Client session handling → `session.rs`
   - Bot management → `bot_manager.rs`
   - Core state structures → `state.rs`

#### Native Backend: eval.rs Refactoring
**Target:** Split 443-line file into logical components

1. **Create `native_mcg/src/poker/` directory:**
   ```
   poker/
   ├── mod.rs
   ├── evaluation.rs    # Hand evaluation algorithms
   ├── cards.rs         # Card representation and utilities
   ├── hand_ranking.rs  # Hand ranking logic
   └── constants.rs     # Game constants and configurations
   ```

#### Frontend: qr_scanner.rs Refactoring
**Target:** Split 389-line file into UI and logic components

1. **Create `frontend/src/camera/` directory:**
   ```
   camera/
   ├── mod.rs
   ├── qr_scanner.rs   # QR scanning popup UI
   ├── camera_handler.rs # Camera management
   └── qr_parser.rs    # QR code parsing logic
   ```

### Phase 2: Improve Module Organization

#### Frontend Restructuring
**Target:** Create clearer separation between UI and game logic

1. **Restructure `frontend/src/`:**
   ```
   frontend/src/
   ├── app.rs           # Main application framework
   ├── router.rs        # (unchanged)
   ├── store.rs         # (unchanged)
   ├── ui/              # Pure UI components
   │   ├── mod.rs
   │   ├── screens/     # Move existing screens here
   │   └── components/  # Reusable UI components
   ├── game/            # Game-specific logic
   │   ├── mod.rs
   │   ├── poker/       # Poker-specific game logic
   │   ├── card_games/  # Generic card game logic
   │   └── state/       # Game state management
   ├── camera/          # QR/camera functionality
   │   └── (as above)
   └── lib.rs           # Main entry point
   ```

2. **Move and rename files:**
   - `game.rs` → `app.rs` (main application framework)
   - Move all screens to `ui/screens/`
   - Create reusable component library in `ui/components/`

#### Native Backend Restructuring
**Target:** Better separation of concerns between server, game, and CLI

1. **Restructure `native_mcg/src/`:**
   ```
   native_mcg/src/
   ├── server/          # Core server functionality
   │   ├── mod.rs
   │   ├── state.rs     # (from refactored state.rs)
   │   ├── lobby.rs     # (from refactored state.rs)
   │   ├── transports/  # HTTP, WebSocket, iroh
   │   └── handlers/    # Message handlers
   ├── poker/           # Poker game logic
   │   ├── mod.rs
   │   ├── engine.rs    # Game engine
   │   ├── rules.rs     # Poker rules and validation
   │   └── evaluation.rs # Hand evaluation
   ├── cli/             # Command-line interface
   │   ├── mod.rs
   │   ├── commands/    # CLI command implementations
   │   └── output/      # Pretty printing and formatting
   ├── bots/            # Bot implementation
   │   ├── mod.rs
   │   ├── strategy.rs  # Bot decision making
   │   └── manager.rs  # Bot lifecycle
   └── lib.rs
   ```

2. **Move and rename files:**
   - `backend/` → `server/`
   - `game/` → `poker/` (since it's poker-specific)
   - `pretty.rs` → `cli/output.rs`
   - `eval.rs` → `poker/evaluation.rs`
   - Extract bot logic from `bot.rs` into `bots/` modules

#### Shared Crate Restructuring
**Target:** Better organization of shared types and communication

1. **Restructure `shared/src/`:**
   ```
   shared/src/
   ├── protocol/         # Game protocol messages
   │   ├── mod.rs
   │   ├── messages.rs   # ClientMsg, ServerMsg enums
   │   └── events.rs     # ActionEvent, GameAction enums
   ├── game/             # Game-specific types
   │   ├── mod.rs
   │   ├── state.rs      # GameStatePublic, PlayerPublic
   │   ├── cards.rs      # Card, HandRank types
   │   └── actions.rs    # PlayerAction, Stage enums
   ├── crypto/           # Cryptographic communication
   │   ├── mod.rs
   │   └── communication.rs # Existing crypto types
   └── lib.rs
   ```

### Phase 3: Naming and Interface Improvements

#### Better Naming Conventions
1. **Frontend:**
   - Rename `poker_online.rs` → `poker_screen.rs` (after refactoring)
   - Rename `screens/` submodules to be more descriptive (e.g., `poker_online.rs` → `poker.rs`)

2. **Native Backend:**
   - Rename `eval.rs` → `hand_evaluation.rs`
   - Rename `pretty.rs` → `output_formatter.rs`
   - Rename `game/` → `poker/` (since it's poker-specific)

3. **Shared:**
   - Split `lib.rs` into focused modules by domain
   - Group related types together

#### Interface Improvements
1. **Frontend:**
   - Create consistent screen interface patterns
   - Extract common UI components into reusable modules
   - Separate data models from UI representation

2. **Native Backend:**
   - Define clear interfaces between server and game logic
   - Create transport-agnostic message handling
   - Separate bot interface from bot implementation

3. **Shared:**
   - Create clear API boundaries between protocol, game, and crypto modules
   - Improve type safety with more specific enums and structs

### Phase 1: Break Down Oversized Files (Concrete Implementation)

**Concrete Extraction Plan:**

**Create `frontend/src/game/screens/poker/` directory:**
```rust
poker/
├── mod.rs                    // Exports all modules
├── screen.rs                 // PokerOnlineScreen (main struct, ~100 lines)
├── player_manager.rs         // Extract lines 167-297, 315-329
├── game_rendering.rs        // Extract lines 346-580
├── name_generator.rs         // Extract lines 670-740
├── connection_manager.rs     // Extract lines 63-87, 609-667
└── ui_components.rs         // Extract lines 124-165, 732-1179

```

#### 1.2 Native Backend: state.rs Refactoring (491 lines)
**Current Structure Analysis:**
- **Lines 19-29:** `AppState` struct (shared state container)
- **Lines 47-59:** `Lobby` struct (game state management)
- **Lines 87-491:** Mixed business logic and state operations

**Dependency Mapping:**
```
state.rs is used by:
- backend/mod.rs (re-exports)
- backend/http.rs (AppState for HTTP handlers)
- backend/ws.rs (AppState for WebSocket handlers)
- backend/iroh.rs (AppState for Iroh transport)
- main.rs (AppState creation)
- tests/integration_ws.rs (testing)
```

**Concrete Extraction Plan:**

**Create `native_mcg/src/server/` directory:**
```rust
server/
├── mod.rs                    // Re-export from modules
├── state.rs                  // Core structs only (~100 lines)
├── game_ops.rs               // Game operations (lines 87-123, 165-177, 182-226, 343-361)
├── bot_driver.rs             // Bot management (lines 365-491)
├── broadcast.rs              // State broadcasting (lines 125-133, 141-161, 229-236)
└── message_handlers.rs       // Message processing (lines 239-314)
```

#### 1.3 Native Backend: eval.rs Refactoring (443 lines)
**Current Structure Analysis:**
- **Lines 1-14:** Constants and type definitions
- **Lines 15-50:** Card utility functions
- **Lines 46-443:** Hand evaluation algorithms

**Concrete Extraction Plan:**

**Create `native_mcg/src/poker/` directory:**
```rust
poker/
├── mod.rs                    // Poker-specific logic
├── cards.rs                  // Card types and utilities (lines 1-50)
├── evaluation.rs             // Hand evaluation algorithms (lines 46-443)
└── constants.rs              // Game constants and configurations
```

**Module Responsibilities:**
- **`cards.rs`:** `CardRank`, `CardSuit` enums, `card_rank()`, `card_suit()`, `card_str()`
- **`evaluation.rs`:** `evaluate_best_hand()` and supporting algorithms
- **`constants.rs`:** `NUM_SUITS`, `NUM_RANKS`, `RANK_COUNT_ARRAY_SIZE`


**File Movements:**
1. **Extract routing logic from `game.rs`:**
   - Move `current_screen_path`, `screens`, `screen_registry` to `routing/`
   - Move router-related methods to `routing/router.rs`

2. **Extract UI components from `game.rs`:**
   - Move `settings_open`, `pending_settings`, `Settings` struct to `ui/settings.rs`
   - Move top bar rendering to `ui/top_bar.rs`

3. **Simplify `game.rs`:**
   ```rust
   pub struct App {
       routing: routing::AppRouter,
       ui: ui::AppUI,
       state: state::AppState,
   }
   ```

### Phase 3: Interface Improvements

**Frontend:**
- Create consistent `ScreenWidget` interface patterns
- Extract common button styling helpers
- Create generic table/list rendering components
- Separate data models from UI representation

**Native Backend:**
- Define clear `ServerState` trait interface
- Create transport-agnostic message handling
- Extract configuration management interface

**Shared:**
- Create clear API boundaries between protocol, game, and crypto
- Use more specific enums and structs for type safety
