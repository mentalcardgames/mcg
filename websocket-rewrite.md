# WebSocket Handling Rewrite Plan

## Current Implementation Analysis

The current WebSocket implementation in `frontend/src/game/connection.rs` suffers from several architectural issues:

### Problems Identified:

1. **Inconsistent Architecture**: The code claims to be "event-driven" but actually uses a polling-based approach with message queuing.

2. **Message Queuing Overhead**:
   - Messages are buffered in a `VecDeque<ServerMsg>` via `event_queue`
   - Requires `poll_messages()` to be called in the UI loop to process messages
   - Adds unnecessary latency between message receipt and processing

3. **Missing UI Repaint Trigger**:
   - Despite comments mentioning "request UI repaints", no `request_repaint()` calls are actually made
   - UI only updates during the next egui frame, not immediately when messages arrive

4. **Complex Closure Management**:
   - Multiple closure clones for different event handlers
   - Complex `Rc<RefCell<_>>` sharing pattern
   - Potential memory leaks if not managed carefully

5. **Tight Coupling**:
   - Connection service doesn't have a clean interface for message handling
   - UI code must call `poll_messages()` and manually apply messages to state

## Proposed Simplified Architecture

### Core Principles:
1. **Immediate Processing**: Handle incoming messages immediately without queuing
2. **Direct UI Updates**: Trigger immediate UI repaint when messages arrive
3. **Simple Interface**: Clean `handle_msg` and `send_msg` functions
4. **Decoupled Design**: Separate connection management from message processing

### New Design:

```rust
pub struct WebSocketConnection {
    ws: Option<WebSocket>,
    on_message: Option<Box<dyn Fn(ServerMsg)>>,
    on_error: Option<Box<dyn Fn(String)>>,
    on_close: Option<Box<dyn Fn(String)>>,
    // Closure handles for cleanup
    _onmessage: Option<Closure<dyn FnMut(MessageEvent)>>,
    _onerror: Option<Closure<dyn FnMut(Event)>>,
    _onclose: Option<Closure<dyn FnMut(CloseEvent)>>,
}

impl WebSocketConnection {
    pub fn new() -> Self { ... }

    pub fn connect(
        &mut self,
        server_address: &str,
        on_message: impl Fn(ServerMsg) + 'static,
        on_error: impl Fn(String) + 'static,
        on_close: impl Fn(String) + 'static,
    ) { ... }

    pub fn send_msg(&self, msg: &ClientMsg) { ... }

    pub fn close(&mut self) { ... }
}
```

### Integration Pattern:

```rust
// In PokerOnlineScreen
impl PokerOnlineScreen {
    fn new() -> Self {
        let mut screen = Self { ... };

        screen.conn.connect(
            &screen.edit_server_address,
            move |msg: ServerMsg| {
                // handle_msg - immediate processing
                app_state.apply_server_msg(msg);
                ctx.request_repaint(); // immediate UI update
            },
            move |error: String| {
                // handle error
                app_state.last_error = Some(error);
                ctx.request_repaint();
            },
            move |reason: String| {
                // handle close
                app_state.connection_status = ConnectionStatus::Disconnected;
                ctx.request_repaint();
            }
        );

        screen
    }

    fn send_msg(&self, msg: &ClientMsg) {
        self.conn.send_msg(msg);
    }
}
```

## Implementation Plan

### Phase 1: Create New WebSocket Service
1. Create `WebSocketConnection` struct with simplified design
2. Implement direct message processing (no queuing)
3. Add proper `request_repaint()` calls in message handlers
4. Maintain existing WebSocket lifecycle management

### Phase 2: Update PokerOnlineScreen
1. Replace `ConnectionService` with `WebSocketConnection`
2. Remove `poll_messages()` calls from UI loop
3. Implement direct message handling callbacks
4. Add `send_msg` convenience method

### Phase 3: Remove Old Implementation
1. Delete `ConnectionService`
2. Update imports and references
3. Test end-to-end functionality

## Benefits of New Design:

1. **Immediate Response**: Messages processed instantly, no polling delay
2. **Simpler Code**: No queuing, no complex `Rc<RefCell>` patterns
3. **Better Performance**: Reduced memory allocations and copying
4. **Cleaner API**: Clear `handle_msg`/`send_msg` interface
5. **Correct UI Updates**: Immediate repaints when messages arrive
6. **Better Testability**: Easier to mock and test message handling

## Feasibility Assessment

**✅ Highly Feasible** - The proposed design is:
- Architecturally sound
- Compatible with existing `AppState` and egui patterns
- Removes complexity while maintaining functionality
- Follows standard WebSocket best practices
- No breaking changes to the backend protocol

The rewrite addresses all current issues while providing a cleaner, more maintainable implementation that matches the user's requirements exactly.