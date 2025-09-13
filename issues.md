# Code Architecture Analysis Report

## Executive Summary

The MCG project is a well-structured Rust workspace implementing a browser-based card game with WASM frontend and native backend. The codebase demonstrates good separation of concerns with clear layering between frontend, backend, and shared protocol types. However, several architectural issues, error handling inconsistencies, and potential maintenance concerns have been identified that could lead to bugs or technical debt.

## Critical Issues

## Architectural Concerns

### 5. **Complex State Management in Connection Service**
**Location**: `/home/benjamin/workspace/mcg/frontend/src/game/connection.rs:15-18`
**Impact**: Medium - Difficult to reason about state flow
**Issue**: The connection service mixes WebSocket management with message queuing and event handling, making it difficult to track state transitions.

**Suggested Fix**: Separate concerns into distinct components: connection manager, message queue, and event dispatcher.

## Logic Errors


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

