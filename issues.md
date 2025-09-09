# Code Architecture Analysis Report

## Executive Summary

The MCG project is a well-structured Rust workspace implementing a browser-based card game with WASM frontend and native backend. The codebase demonstrates good separation of concerns with clear layering between frontend, backend, and shared protocol types. However, several architectural issues, error handling inconsistencies, and potential maintenance concerns have been identified that could lead to bugs or technical debt.

## Critical Issues

### 1. **Excessive Use of `unwrap()` and `expect()` Calls**
**Location**: Multiple files throughout codebase
**Impact**: High - Potential runtime panics
**Files Affected**:
- `frontend/src/game/card.rs:52,54`
- `frontend/src/game/field.rs:192,253,276`
- `frontend/src/qr_scanner.rs:81,82,88,98`
- `frontend/src/lib.rs:78,80`
- `native_mcg/src/bin/cli/utils.rs:10`
- `native_mcg/src/game/flow.rs:197,202,210,215,217`
- `native_mcg/tests/integration_ws.rs:55`

**Issue**: The codebase contains numerous `unwrap()` and `expect()` calls that can cause runtime panics, particularly in WASM context where panics are harder to debug.

**Suggested Fix**: Replace with proper error handling using `Result` types and graceful fallbacks.

### 2. **Memory Leaks in WebSocket Closure Management**
**Location**: `/home/benjamin/workspace/mcg/frontend/src/game/connection.rs:86,100,115`
**Impact**: High - Memory leaks in long-running sessions
**Issue**: Closures are explicitly leaked using `forget()` without proper cleanup mechanism, leading to potential memory leaks in long-running browser sessions.

**Suggested Fix**: Implement proper closure lifecycle management or use a different approach for event handlers.

### 3. **Inconsistent Error Handling Patterns**
**Location**: `/home/benjamin/workspace/mcg/native_mcg/src/backend/state.rs:136-196`
**Impact**: Medium - Inconsistent error propagation
**Issue**: Mixed error handling approaches with some functions returning `Result` and others using `Option<String>` for errors.

**Suggested Fix**: Standardize on `anyhow::Result<T>` for consistent error handling.

## Architectural Concerns

### 5. **Complex State Management in Connection Service**
**Location**: `/home/benjamin/workspace/mcg/frontend/src/game/connection.rs:15-18`
**Impact**: Medium - Difficult to reason about state flow
**Issue**: The connection service mixes WebSocket management with message queuing and event handling, making it difficult to track state transitions.

**Suggested Fix**: Separate concerns into distinct components: connection manager, message queue, and event dispatcher.

### 6. **Bot Logic Embedded in Backend State**
**Location**: `/home/benjamin/workspace/mcg/native_mcg/src/backend/state.rs:308-436`
**Impact**: Medium - Violation of single responsibility
**Issue**: Bot AI logic is mixed with state management, making the backend harder to test and maintain.

**Suggested Fix**: Extract bot logic into a separate module with clear interfaces.

## Logic Errors

### 7. **Potential Race Condition in Bot Driving**
**Location**: `/home/benjamin/workspace/mcg/native_mcg/src/backend/state.rs:357-411`
**Impact**: High - Concurrent state modification issues
**Issue**: The bot driver rechecks conditions after acquiring write lock but doesn't handle cases where state changes between check and action.

**Suggested Fix**: Use atomic operations or implement proper state locking strategies.

### 8. **Inconsistent Player ID Handling**
**Location**: `/home/benjamin/workspace/mcg/native_mcg/src/backend/state.rs:152-196`
**Impact**: Medium - Potential player confusion
**Issue**: Player IDs are converted between `usize` and `PlayerId` types inconsistently, leading to potential indexing errors.

**Suggested Fix**: Standardize on `PlayerId` type throughout the backend and provide conversion utilities.


## Code Quality Issues

### 11. **Large Function with Multiple Responsibilities**
**Location**: `/home/benjamin/workspace/mcg/native_mcg/src/backend/state.rs:212-285`
**Impact**: Medium - Difficult to test and maintain
**Issue**: `handle_client_msg` function handles multiple message types with complex logic for each.

**Suggested Fix**: Split into separate handler functions for each message type.

## Maintainability Issues

### 18. **Inconsistent Module Organization**
**Location**: `/home/benjamin/workspace/mcg/frontend/src/game/screen.rs:1-2`
**Impact**: Low - Navigation confusion
**Issue**: Some modules are just re-exports while others contain actual implementation.

**Suggested Fix**: Standardize module organization or document the rationale for re-exports.

## Suggested Fixes

### Immediate Actions (High Priority)
1. **Replace `unwrap()` calls**: Systematically replace all `unwrap()` and `expect()` calls with proper error handling
2. **Fix WebSocket memory leaks**: Implement proper closure lifecycle management
3. **Standardize error handling**: Choose a consistent error handling pattern across the codebase

### Short-term Actions (Medium Priority)
4. **Extract bot logic**: Move bot AI logic to a separate module
5. **Fix race conditions**: Implement proper synchronization in bot driving logic

