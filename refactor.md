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

4. **`frontend/src/qr_scanner.rs`** (389 lines)
   - Contains QR code scanning UI, camera handling, and parsing logic
   - UI and parsing logic should be separated

### Misplaced Components (Priority: Medium)

#### Frontend Issues:
1. **`frontend/src/game.rs`** (319 lines)
   - Contains App struct, router integration, and screen management
   - Should separate application framework from game-specific logic

2. **`frontend/src/game/field.rs`** (339 lines)
   - Contains card field rendering, drag-drop logic, and card utilities
   - UI rendering and interaction logic should be separated

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

#### Shared Crate:
- Single `lib.rs` contains all protocol definitions (187 lines)
- Cryptographic communication mixed with game protocol
- No clear separation of concerns

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
   - Extract game logic from `field.rs` into `game/` modules
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
   - Rename `field.rs` → `card_field.rs` or `table_view.rs`
   - Rename `screens/` submodules to be more descriptive (e.g., `poker_online.rs` → `poker.rs`)

2. **Native Backend:**
   - Rename `eval.rs` → `hand_evaluation.rs`
   - Rename `pretty.rs` → `output_formatter.rs`
   - Rename `backend/` → `server/`
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

## Implementation Strategy

### Order of Operations
1. **Phase 1a:** Break down `poker_online.rs` (highest impact)
2. **Phase 1b:** Refactor `state.rs` (critical for server architecture)
3. **Phase 1c:** Split `eval.rs` (improves poker logic organization)
4. **Phase 2:** Reorganize module structures
5. **Phase 3:** Naming and interface improvements

## Detailed Implementation Plans

### Phase 1: Break Down Oversized Files (Concrete Implementation)

#### 1.1 Frontend: poker_online.rs Refactoring (1,179 lines)
**Current Structure Analysis:**
- **Lines 13-24:** `PokerOnlineScreen` struct with mixed concerns
- **Lines 26-581:** Core implementation with 21+ UI methods
- **Lines 583-741:** Secondary implementation with name generation
- **Lines 743-789:** Trait implementations
- **Lines 792-1179:** 22 standalone helper functions

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

**Module Responsibilities:**
- **`screen.rs`:** Main struct, constructor, trait implementations
- **`player_manager.rs`:** Player setup, table rendering, player actions
- **`game_rendering.rs`:** Game state rendering, action buttons, panels
- **`name_generator.rs`:** Random name generation (6 pure functions)
- **`connection_manager.rs`:** WebSocket connection management (4 methods)
- **`ui_components.rs`:** Reusable UI helpers (card_chip, formatting functions)

**Implementation Steps:**
1. Create `player_manager.rs` with extracted methods
2. Update imports in `screen.rs` to use new modules
3. Test each extraction incrementally
4. Repeat for other modules

**Expected Result:** Main screen reduced to ~150 lines, focused modules with single responsibilities

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

**Module Responsibilities:**
- **`state.rs`:** `AppState`, `Lobby` structs, basic accessors
- **`game_ops.rs`:** `create_new_game()`, `validate_and_apply_action()`, `start_new_hand_and_print()`
- **`bot_driver.rs`:** `drive_bots_with_delays()`, bot decision logic
- **`broadcast.rs`:** `current_state_public()`, `broadcast_state()`, `broadcast_and_drive()`
- **`message_handlers.rs`:** All `handle_*` functions as thin wrappers

**Implementation Steps:**
1. Extract `game_ops.rs` first (least dependencies)
2. Extract `broadcast.rs` (depends only on state)
3. Extract `message_handlers.rs` (uses both game_ops and broadcast)
4. Extract `bot_driver.rs` (most complex, depends on everything)
5. Keep `state.rs` with minimal core functionality

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

### Phase 2: Module Reorganization (Concrete Implementation)

#### 2.1 Frontend Restructuring
**Current Issues Identified:**
- `game.rs` (320 lines) acts as god object - routing + settings + screen management
- `field.rs` (340 lines) mixes UI rendering with game logic
- Tight coupling between `field.rs` and screens via `DNDSelector`

**New Structure Plan:**
```rust
frontend/src/
├── app.rs                   // Main eframe::App implementation (~150 lines)
├── components/             // Reusable UI components
│   ├── mod.rs
│   ├── card_field.rs       // Extract UI from field.rs
│   ├── drag_drop.rs        // Generic D&D logic
│   └── theme.rs            // Theme management
├── state/                  // Pure state management
│   ├── mod.rs
│   ├── app_state.rs        // Extract from store.rs
│   ├── actions.rs          // Action types
│   └── reducers.rs         // State handlers
├── routing/               // Navigation management
│   ├── mod.rs
│   ├── router.rs           // Extract from game.rs
│   └── screen_registry.rs  // Extract from game.rs
├── ui/                    // App-specific UI
│   ├── mod.rs
│   ├── settings.rs         // Extract from game.rs
│   └── top_bar.rs          // Extract from game.rs
└── game/                  // Existing game logic (simplified)
    ├── screens/            // (unchanged)
    └── field.rs            // Simplified to game logic only
```

**File Movements:**
1. **Extract routing logic from `game.rs`:**
   - Move `current_screen_path`, `screens`, `screen_registry` to `routing/`
   - Move router-related methods to `routing/router.rs`

2. **Extract UI components from `game.rs`:**
   - Move `settings_open`, `pending_settings`, `Settings` struct to `ui/settings.rs`
   - Move top bar rendering to `ui/top_bar.rs`

3. **Split `field.rs`:**
   - UI rendering → `components/card_field.rs`
   - Game logic (push, pop, insert) → keep in `game/field.rs`
   - Drag drop logic → `components/drag_drop.rs`

4. **Simplify `game.rs`:**
   ```rust
   pub struct App {
       routing: routing::AppRouter,
       ui: ui::AppUI,
       state: state::AppState,
   }
   ```

#### 2.2 Native Backend Restructuring
**Current Issues:**
- `backend/` mixes server infrastructure with game logic
- `game/` is poker-specific but not clearly named
- `pretty.rs` mixes CLI formatting with server concerns

**New Structure Plan:**
```rust
native_mcg/src/
├── server/                // Core server functionality
│   ├── mod.rs
│   ├── state.rs           // (from refactored state.rs)
│   ├── game_ops.rs        // (from refactored state.rs)
│   ├── bot_driver.rs      // (from refactored state.rs)
│   ├── broadcast.rs       // (from refactored state.rs)
│   ├── message_handlers.rs // (from refactored state.rs)
│   └── transports/        // HTTP, WebSocket, iroh
│       ├── mod.rs
│       ├── http.rs        // (move from backend/)
│       ├── ws.rs          // (move from backend/)
│       └── iroh.rs        // (move from backend/)
├── poker/                 // Poker-specific game logic
│   ├── mod.rs
│   ├── engine.rs          // Game engine
│   ├── rules.rs           // Poker rules
│   ├── evaluation.rs      // (from refactored eval.rs)
│   ├── cards.rs           // (from refactored eval.rs)
│   └── constants.rs       // (from refactored eval.rs)
├── cli/                   // Command-line interface
│   ├── mod.rs
│   ├── commands.rs        // CLI commands
│   ├── output.rs          // (rename pretty.rs)
│   └── bin/
│       ├── mcg-cli.rs     // (move from bin/)
│       └── cli/
│           └── (existing CLI structure)
├── bots/                  // Bot implementation
│   ├── mod.rs
│   ├── strategy.rs        // Bot decision making
│   └── manager.rs         // Bot lifecycle
└── lib.rs
```

**File Movements:**
1. **Move `backend/` → `server/`**
2. **Move `game/` → `poker/`** (since it's poker-specific)
3. **Rename `pretty.rs` → `cli/output.rs`**
4. **Create `bots/` directory** and extract bot logic from `bot.rs`

#### 2.3 Shared Crate Restructuring
**Current Issues:**
- `lib.rs` (187 lines) contains all protocol definitions
- Cryptographic communication mixed with game protocol
- No clear separation of concerns

**New Structure Plan:**
```rust
shared/src/
├── protocol/              // Game protocol messages
│   ├── mod.rs
│   ├── messages.rs        // ClientMsg, ServerMsg
│   └── events.rs          // ActionEvent, GameAction
├── game/                  // Game-specific types
│   ├── mod.rs
│   ├── state.rs           // GameStatePublic, PlayerPublic
│   ├── cards.rs           // Card, HandRank types
│   ├── actions.rs         // PlayerAction, Stage enums
│   └── constants.rs       // Game constants
└── crypto/                // Cryptographic communication
    ├── mod.rs
    └── communication.rs   // (existing crypto types)
```

**Content Extraction from lib.rs:**
- **`protocol/messages.rs`:** `ClientMsg`, `ServerMsg` enums
- **`protocol/events.rs`:** `ActionEvent`, `GameAction`, `ActionKind`
- **`game/state.rs`:** `GameStatePublic`, `PlayerPublic`, `PlayerConfig`
- **`game/cards.rs`:** `Card`, `HandRank`, `HandRankCategory`, `HandResult`
- **`game/actions.rs`:** `PlayerAction`, `Stage`, `BlindKind`
- **`game/constants.rs`:** `CARD_NATURAL_SIZE`, `PlayerId` conversion impls

### Phase 3: Interface Improvements

#### 3.1 Better Naming Conventions
**Frontend:**
- `poker_online.rs` → `poker_screen.rs` (after refactoring)
- `field.rs` → `card_field.rs` (UI part) + `game_logic.rs` (logic part)
- `screens/poker_online.rs` → `screens/poker.rs`

**Native Backend:**
- `eval.rs` → `poker/evaluation.rs`
- `pretty.rs` → `cli/output_formatter.rs`
- `backend/` → `server/`
- `game/` → `poker/`

**Shared:**
- Group related types in focused modules
- Improve enum naming consistency

#### 3.2 Interface Improvements
**Frontend:**
- Create consistent `ScreenWidget` interface patterns
- Extract common button styling helpers
- Create generic table/list rendering components
- Separate data models from UI representation

**Native Backend:**
- Define clear `ServerState` trait interface
- Create transport-agnostic message handling
- Separate `BotDriver` trait from bot implementations
- Extract configuration management interface

**Shared:**
- Create clear API boundaries between protocol, game, and crypto
- Use more specific enums and structs for type safety
- Add builder patterns for complex types

### Risk Mitigation
1. **Incremental Changes:** Refactor one file at a time with tests
2. **Maintain API Compatibility:** Keep existing public interfaces during transition
3. **Comprehensive Testing:** Ensure all functionality is preserved
4. **Documentation:** Update module documentation as changes are made

### Success Metrics
1. No single source file exceeds 400 lines
2. Clear module boundaries with single responsibility
3. Improved code discoverability and maintainability
4. Reduced coupling between modules
5. Better test coverage due to smaller, focused modules

## Timeline Estimate with Concrete Tasks

### Phase 1: Break Down Large Files (4-5 days)
- **1.1 poker_online.rs refactor:** 2 days
  - Day 1: Extract `player_manager.rs` and `game_rendering.rs`
  - Day 2: Extract remaining modules and test integration

- **1.2 state.rs refactor:** 1.5 days
  - Day 1: Extract `game_ops.rs` and `broadcast.rs`
  - Day 2: Extract `bot_driver.rs` and `message_handlers.rs`

- **1.3 eval.rs refactor:** 0.5 days
  - Split into `cards.rs`, `evaluation.rs`, `constants.rs`

### Phase 2: Module Reorganization (3-4 days)
- **2.1 Frontend restructuring:** 2 days
  - Day 1: Extract routing and UI components from `game.rs`
  - Day 2: Split `field.rs` and create component modules

- **2.2 Backend restructuring:** 1 day
  - Move `backend/` → `server/`, rename modules, reorganize

- **2.3 Shared crate restructuring:** 0.5 days
  - Split `lib.rs` into focused modules

### Phase 3: Interface Improvements (1-2 days)
- **3.1 Naming improvements:** 0.5 days
- **3.2 Interface standardization:** 1 day
- **3.3 Documentation updates:** 0.5 days

### Testing and Validation (1-2 days)
- Integration testing, build verification, functionality testing

**Total Estimated Time:** 9-13 days

### Success Metrics with Concrete Targets

1. **File Size Reduction:**
   - No source file exceeds 400 lines (currently 1,179 line max)
   - Average file size under 200 lines
   - `poker_online.rs` reduced from 1,179 → ~150 lines
   - `state.rs` reduced from 491 → ~100 lines

2. **Module Boundaries:**
   - Clear single responsibility for each module
   - Circular dependencies eliminated
   - Module coupling reduced (measured by import count)
   - Interface segregation applied (traits focused on specific capabilities)

3. **Code Quality:**
   - Test coverage maintained or improved
   - Build times reduced (due to smaller compilation units)
   - Code duplication eliminated
   - God object pattern eliminated

4. **Maintainability:**
   - New feature development time reduced
   - Bug fix localization improved
   - Onboarding time for new developers reduced
   - Code review efficiency improved

## Conclusion

This refactoring plan addresses the major organizational issues in the MCG project while maintaining functionality. The result will be a more maintainable, testable, and understandable codebase with clear module boundaries and better separation of concerns.