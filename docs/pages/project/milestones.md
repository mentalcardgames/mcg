---
outline: deep
---

# Student Milestones

This lists the active milestones and software engineering packages available
for student projects.
Each milestone represents a core challenge within our architecture.
Rather than being tied to rigid academic ECTS levels, tasks are dynamically
scaled and adjusted based on a student's expected workload.

## Miscellaneous

::: danger

TODO: not finalized

Anpassen an [Synchronous Controller Environment](#synchronous-controller-environment)

:::

Coupling game rules directly with system logic limits modularity and prevents
rule reuse.
To support a generic framework, the game engine must be isolated from the
application's infrastructure, allowing it to evaluate rules independent.

### Game Engine Crate Extraction

**The Problem:**

The current state machine logic for static Poker is tightly coupled with
frontend rendering and backend connection supervisors, causing it to be active
at all times.

**Motivation:**

We would like to have the poker implementation decoupled and controllable.
This way it can be run only when needed and it allows us to compare a dynamic
implementation of poker in CGDL.

**Deliverables:**

- Extract all Poker-specific states, rules, and game FSM code into a dedicated
workspace crate (e.g., `crates/static_poker` or `crates/static/poker`).
- Define a clean, uniform interface/traits that the main application can call to
step through states.
- Ensure the existing Poker game remains fully playable using the new decoupled
boundaries.

### Synchronous Controller Environment

**The Problem:**

Currently, the codebase lacks a dedicated controller, making it difficult to
grasp the backend architecture as there is no single point where all
coordination and state management are located.
Furthermore, this concurrency forces developers to write complex async Rust for
logically synchronous operations.

**Motivation:**

Introducing a dedicated synchronous controller running sequentially in a
single-threaded execution core eliminates concurrency issues such as deadlocks
and race conditions.
Additionally, a synchronous setting is much more familiar and intuitive to
develop in, sparing students from the complexities of async task management and
compiler lifetime friction.

**Deliverables:**

- Introduce a dedicated, synchronous **Controller** struct running in a
sequential, synchronous execution setting.
- Spawn a dedicated OS thread to execute the Controller's sequential loop.
- Establish an MPSC queue to read incoming events from the async network shell
to the Controller's thread.
- Design the Controller to manage the core application state, holding state
directly or delegating ownership to appropriate underlying models
(such as `Lobby`, `Game`, or `Engine`).

### Actor-Inspired Backend Messaging Architecture

**The Problem:**

The native backend has to concurrently manage multiple connection channels
without blocking other calculations.
Currently, these asynchronous connection tasks share state via locks,
potentially causing deadlocks, and handling more than only connection related
jobs.

**Motivation:**

To align connection handling with the asynchronous network shell, tasks should
act as lightweight communication actors that feed the synchronous Controller
without direct access to the state or locking constraints.

**Deliverables:**

- Restructure connection listeners as lightweight async actor tasks.
- Ensure all connection actors parse raw incoming buffers into typed messages
(`Frontend2BackendMsg`, `Peer2PeerMsg`) and route them to the central Controller
over a MPSC channel.
- Implement a direct messaging mechanism enabling the Controller to send
outgoing packets to specific open connections independently through the network
layer.
- Allow the Controller to dynamically manage connection lifecycles by initiating
new connections and closing existing ones.

### Frontend Encapsulated & Event-Driven Networking

**The Problem:**

Connection lifecycle decisions are distributed across screens.
Screens can initiate or close the shared connection through
`FrontendInterface`, while the default backend address remains hardcoded
instead of being derived from the browser location.
Connection errors and close reasons are only logged and are not represented as
application-visible state.

`FrontendInterface` also exposes the general-purpose `send_msg` method.
Although it accepts a typed `Frontend2BackendMsg`, screens remain coupled to
the complete wire-protocol enum instead of using purpose-specific frontend
operations.

**Motivation:**

A clean interface boundary should hide transport details and connection
lifecycle policy behind high-level, typed application operations.
This keeps screens independent of the underlying WebSocket implementation and
provides a single place for reconnect behavior, connection feedback, and
reactive UI updates.

**Deliverables:**

- Derive the default backend host, port, and WebSocket scheme from the browser
location instead of defaulting to `127.0.0.1:3000`.
- Move connection startup, reconnect, and shutdown policy into `FrontendApp` so
screens request application operations rather than managing lifecycle timing.
- Replace general-purpose protocol access with purpose-specific methods where a
stable frontend operation exists; retain a narrow sender abstraction only where
generic message forwarding is intentional.
- Represent connection status, errors, and close reasons as application-visible
state and request a repaint for error/close callbacks.

### Frontend State Decoupling

**The Problem:**

`FrontendInterface::state()` and `state_mut()` expose the complete
`FrontendState`, whose fields are public.
Any screen can therefore modify application-owned settings and the screen
registry directly.
The lifetime policy for screen instances and their local state during
navigation is also not formally defined or tested.

**Motivation:**

Narrow application capabilities make ownership explicit and keep screens
modular.
Screens should request navigation or configuration changes through
purpose-specific operations without receiving unrestricted mutable access to
global state.
A defined screen-lifetime policy also makes local state behavior predictable
when users navigate away and return.

**Deliverables:**

- Make `FrontendState` fields private and remove unrestricted `state_mut()`
access from screens.
- Add purpose-specific getters and commands for player identity, server
configuration, theme/DPI settings, and any other legitimate global capability.
- Keep registry mutation and applied UI settings entirely under `FrontendApp`
ownership.
- Document and test the lifetime policy for screen instances and their local
state when navigating away and returning.

## UI Rendering

Card games vary wildly in their rules, spatial layouts, hidden information, and
card interactions.
To support arbitrary games, the rendering layer must dynamically adapt to
different game states rather than relying on hardcoded heuristics or assumptions
about a single game's structure.

### Text-based User Interface (TUI) Frontend

**The Problem:**

Directly implementing an interface capable of rendering any arbitrary card game
is extremely difficult, as they vary wildly in rules, spatial layouts,
hidden information, and card interactions.
Additionally, our current graphical frontend is hardcoded specifically for the
static Poker game, making it impossible to render other games.
Furthermore, the existing command-line interface (`mcg-cli`) historically
relied on HTTP request-response endpoints, which have been removed from the
backend, and lacked persistent streaming required for reactive terminal
experiences.

**Motivation:**

A Text-based User Interface (TUI) bypasses layout and visual styling
complexities by rendering structured text or ASCII grids.
Following our **split-node thin-client** model, the TUI operates as a
standalone View client that connects to the native backend's Controller via
WebSocket (`/ws`), receiving pushed state projections and sending user actions
as standard message packets in real time.
Migrating the CLI and TUI tooling to WebSocket provides a unified, persistent
duplex connection matching the web frontend, eliminating polling and enabling
instant reactivity for game events.
This serves as a first stepping stone to test and verify the CGDL engine's
capabilities before building a full GUI.

**Deliverables:**

- Implement a terminal-based frontend View binary either as part of the
`mcg-cli` binary or as a standalone crate.
- Migrate `mcg-cli`'s default transport from legacy HTTP to WebSocket
(`ws://localhost:3000/ws`), establishing persistent duplex communication with
the `/ws` endpoint and adopting the `mcg.frontend` subprotocol.
- Connect to the native backend controller using the WebSocket messaging
protocol (`Frontend2BackendMsg` / `Backend2FrontendMsg`), listening for pushed
state updates reactively instead of polling.
- Determine all components from CGDL that need representation.
- Provide implementations of
[`std::fmt::Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html)
or a custom trait for all representable components.
- Provide an interactive prompt allowing the user to select which parts of the
received state projection to print or track in real time.
- Support shortcuts or commands to quickly display specific zones, variables,
or parts of the game state.
- Implement a "watchlist" mechanism to pin chosen components, keeping them
permanently visible and auto-updating them on the screen when the state changes.

### Providing more widgets

**The Problem:**

The frontend crate currently implements only basic card and field rendering
primitives (e.g., `SimpleField`).
They are insufficient for rendering all components found in card games, such as
grids, multiplayer tables, and additional player attributes.
Furthermore, they lack generic interfaces for advanced interactions like
multi-selection and drag-and-drop between arbitrary fields.

**Motivation:**

Developing a library of reusable, rule-agnostic UI widgets ensures they can be
composed to represent diverse game zones and layouts independently of any
concrete game rules.
Developing these widgets independently ensures they are thoroughly tested,
modularized, and interactive, setting up a solid foundation for the renderer to
easily translate CGDL files into interactive interfaces.

**Deliverables:**

- Expand the widget library in the `frontend` crate to support components from
CGDL.
- Make parameters configurable (e.g., spacing, overlap offset, rotation,
card size, margins).
- Standardize interaction states (e.g., drag-and-drop source/destination tags,
single/multi-selection modes, hover preview/zoom).
- Decouple widgets from concrete game state types so they only rely on generic
traits.
- Provide a widget catalog or demonstration screen in the game client
showcasing all layout widgets and their configurations.

### Extending CGDL with Markup

**The Problem:**

There is an immense variety of card games, and almost every game layout is
unique.
When comparing two different card games, it is nearly guaranteed that they
place, arrange, and move cards across the table in distinct ways
(e.g., community cards in the center, hands held privately, discard piles,
or separate trick-taking zones).
Trying to support infinitely many game layouts without an explicit
markup language would force the rendering system to rely on complex layout
heuristics, which is fragile and difficult to scale.

**Motivation:**

In the physical world, card game rulebooks not only explain the rules but also
specify where the deck, discard piles, and players' hands are physically located
on the table.
We can apply this exact analogy to mental card games.
Since we already have a Card Game Description Language (CGDL) to describe the
rules, we can extend it with markup capabilities that describe the layout and
placement of elements.

**Deliverables:**

- Design the syntax and semantics of the CGDL language extension for spatial
markup definitions.
- Implement parser support for the new layout markup within the existing
CGDL parser.
- Ensure the layout extension integrates seamlessly with the existing CGDL and
game engine.

## Tooling & Developer Experience

Developing custom game descriptions and debugging distributed peer-to-peer state
mutations is difficult and slow without visibility.
High-quality tooling and editor integrations are required to inspect runtime
behaviors and validate rules during development.

### State Mutation & Network Control

**The Problem:**

There are currently no developer tools to observe or control the project's
behavior at runtime.
Debugging peer-to-peer state mutations and component interactions is tedious and
slow, as developers must write custom manual tests to simulate edge cases and
cannot inspect or alter state dynamically.

**Motivation:**

We need a flexible developer playground to debug the native node.
By exposing dedicated debugging endpoints on the backend **Controller**,
developers can query and manipulate game state, craft custom protocol messages,
and control peer connections at runtime to test algorithms and verify execution
flows on the fly.

**Deliverables:**

- Extend the TUI console to act as a debug View client connecting to the
Controller's administrative interface.
- Expose debug endpoints on the Controller to allow safe inspection and mutation
of the underlying Model (game state) at runtime.
- Support crafting and injecting test protocol messages directly into the
Controller's P2P network processing pipeline.
- Expose connection management commands (initiate, disconnect, block) to
simulate network disruptions and P2P topologies.

### CGDL Language Server Protocol (LSP)

**The Problem:**

Developers writing card game descriptions currently lack the full extent of
modern tooling provided by an LSP.
This makes writing CGDL error-prone and frustrating.

**Motivation:**

The goal is to establish a rich developer experience for writing CGDL.
Implementing a Language Server Protocol (LSP) server provides real-time
diagnostics, autocompletion, and tooltips directly inside the developer's
editor, making CGDL self-documenting and safe to write.

**Deliverables:**

- Research and document all standard capabilities specified in the
Language Server Protocol (LSP) specification.
- Formulate a design strategy explaining how those capabilities will be utilized
in the context of CGDL.
- Audit and document existing features in the LSP codebase.
- Establish a prioritized implementation roadmap, ranking features
(e.g., autocomplete, hover tooltips) based on their importance to CGDL
usage scenarios.
- Implement the prioritized LSP capabilities in the language server backend
(`lsp_server` crate) using the CGDL parser.
- Build and package a VS Code extension that bundles and launches the Rust LSP
executable.

## Cryptography & Zero-Knowledge Proofs (ZKP)

::: danger

TODO: not finalized

Adjust for Connection & Message Authentication

:::

To support serverless card games, our project relies on distributed trust.
Instead of relying on a trusted central authority or server to deal cards and
maintain state secrets, we use cryptographic primitives and
Zero-Knowledge Proofs (ZKPs).
This ensures that players can hide their private hands, perform verifiable
operations (such as shuffling or drawing), and prove their compliance with game
rules without revealing any sensitive information.

### Connection & Message Authentication

**The Problem:**

In a decentralized P2P network, game messages are broadcasted and
relayed across multiple untrusted nodes.
Without payload signing, any malicious node in the routing path can tamper with
packets, replay old messages, or impersonate other players.

Furthermore, while message signatures prove that a packet was authored by a
specific cryptographic key, they do not verify *who* owns that key.
A malicious peer can generate a new key pair, connect to the network, and claim
to be a different player.
Without a mechanism to bind cryptographic identities to physical peers, players
remain vulnerable to impersonation and Sybil attacks.

**Motivation:**

To establish a trustless yet secure gaming session, we need a two-tier
authentication stack:

**Message-level Authentication:**
Each node signs its outgoing message payloads using a private key
(e.g., Ed25519).
Relayed messages can then be independently verified by every peer in the
network, ensuring non-repudiation and integrity even if the transport layer is
compromised or messages are relayed through malicious actors.

**Identity & Connection Verification:**
To prevent key-spoofing and MITM attacks, players must be able to verify that
the cryptographic keys used for message signing actually belong to their
intended peers.
By mapping a public key hash or connection fingerprint into a human-readable
representation (such as an Emoji Hash, Identicon, or mnemonic phrase), players
can verify the session's authenticity out-of-band.
This verification does not have to block the connection setup;
it can be performed at any point during the session to confirm the integrity of
the player identities.

**Deliverables:**

- Implement local generation and secure storage of peer key pairs, linking them
to player identities.
- Design and integrate a packet signing protocol.
Every outgoing backend P2P message payload must be signed, and all incoming
messages must be verified against the sender's public key before being processed
by the game engine.
- Incorporate replay prevention mechanisms (e.g., monotonic sequence numbers, timestamps, or nonce challenges) within the signed payload.
- Implement a connection fingerprinting system
(e.g., hashing peer public keys into a deterministic set of emojis, a structured
Identicon, or a mnemonic string).
- Expose the connection fingerprints on both the backend and frontend.
Provide a UI component allowing players to inspect and compare connection
fingerprints at any time.

### Implementing the Toolbox

**The Problem:**

To play card games over a decentralized network without a trusted third party,
players must be able to perform card operations
(such as shuffling, dealing, drawing, and verification) securely.
Currently, our application lacks the core cryptographic primitives needed to
hide card values from players while keeping the game state verifiable and
preventing cheating or collusion.

**Motivation:**

We want to build a general-purpose application capable of playing arbitrary
card games.
To achieve this, we need a toolbox of primitives as defined in
Christian Schindelhauer's paper *A Toolbox for Mental Card Games* (1998)
(referenced in [Literature & References](../organisational/literature.md#foundational-literature)).
Implementing these primitives allows players to hide card information while
maintaining a shared, secure game state.
This can be done either by writing them from scratch in Rust, or by wrapping an
already implemented C/C++ version
(such as the [libtmcg library](https://savannah.nongnu.org/projects/libtmcg/))
inside a Rust crate.
Alternatively, it's possible to implement the improved primitives described by
Heiko Stamer in
[Efficient Electronic Gambling: An Extended Implementation of the Toolbox for Mental Card Games](../organisational/literature.md#foundational-literature).

**Deliverables:**

- Research the foundational primitives described in
*A Toolbox for Mental Card Games* (1998)
and the improved implementations in
*Efficient Electronic Gambling* (2005).
- Provide the implementation either as a native Rust crate written from scratch,
or as a Rust FFI/wrapper around an existing C/C++ implementation like `libtmcg`.
- Expose a clean, idiomatic Rust API that allows for verifiable deck and
card operations.

### Interface for Cryptography

**The Problem:**

Different cryptographic protocols (e.g., ElGamal vs. Lattice Codes) and
implementations offer varying trade-offs in performance, security, and
proof sizes.
Without a unified interface, integrating a specific cryptographic implementation
directly into the project couples the specific logic with that
cryptography library.
This makes studying, benchmarking, or swapping implementations tedious and
error-prone, as it requires modifying the entire integration layer of the
application.

**Motivation:**

By introducing an abstract interface (a set of Rust traits), we decouple the
cryptographic implementation from the engine and networking layers.
Placing this interface in a shared core library allows both the WebAssembly View
(frontend) and the native Controller (backend) to use, serialize, and
deserialize cryptographic objects cleanly.

**Deliverables:**

- Define a set of Rust traits (e.g., in a `mcg-crypto` crate) representing core
mental card game primitives, including key setup, card masking/unmasking,
verifiable shuffles, and ZKP generation/verification.
- Adapt existing implementations to conform to this new unified interface.
- Provide a mock implementation of the traits, which is insecure, to simplify
integration testing and speed up development of other components.

### Protocol Integration

**The Problem:**

In a peer-to-peer system, players broadcast their actions along with
cryptographic proofs to verify that game rules are followed without revealing
private card details.
However, our application currently lacks any protocol integration between the
game engine and the cryptography layer.
Hardcoding cryptographic verification directly inside the game engine's
state machine violates separation of concerns.
We need a modular architecture that separates pure card game logic from the
network-level cryptographic verification and proof-generation.

**Motivation:**

To maintain a clean MVC (Model-View-Controller) structure, the cryptography
layer is implemented as an independent component within our actor model.
The Controller acts as the central orchestrator:
incoming network messages are first intercepted and forwarded to the Crypto
Actor for validation.
The message is only applied to the game engine (the Model) if its ZKP is valid.
Conversely, when a local player mutates their state, the Controller prompts the
Crypto Actor to generate a corresponding ZKP, which is then broadcasted to other
peers.
This prevents code duplication, simplifies testing, and keeps the game engine
decoupled from cryptographic protocol details.

**Deliverables:**

- Implement a dedicated **Crypto Actor** responsible for key management,
card encryption/masking state, and ZKP generation/verification.
- Integrate the verification flow into the MVC Controller:
intercept incoming network messages, request validation from the Crypto Actor,
and forward the message to the game engine only upon successful verification.
- Implement the local action flow:
mutate the local engine state, instruct the Crypto Actor to generate the
necessary ZKP for the mutation, and broadcast the action and proof to the peer
network.

## Network & Communication Infrastructure

::: danger

TODO: not finalized

:::

Since each player runs a local node, the network layer must manage
peer connections, node discovery, and state synchronization.
A clean, asynchronous message pipeline is required to handle high-frequency
communication between the frontend client, the local backend, and remote nodes.

### Reliable Broadcast & Consensus (Forum System)

::: danger

TODO: not finalized

:::

**The Problem:**

How to ensure that when one node broadcasts a state change, the entire mesh network either collectively accepts or vetoes the action?

- **State Propagation:** How to reliably broadcast state transitions across all mesh nodes.
- **Agreement/Veto:** How to coordinate collective acceptance or vetoing of proposed actions.
- **Disconnection:** How to recover and sync consensus state when a node temporarily disconnects.
- **Ordering:** How to handle packet drops and enforce deterministic ordering of concurrent player inputs.

**Motivation:**

**Deliverables:**

- Research and design reliable broadcast and agreement strategies.
- Implement robust retry and confirmation pipelines for direct P2P connections.
- Implement vector clocks or Lamport timestamps for ordering input sequences.
- Specify and implement consensus state recovery plans.

### More Communication Channels

::: danger

TODO: not finalized

:::

**The Problem:**

**Motivation:**

**Deliverables:**

### Decentralized Node Discovery

::: danger

TODO: not finalized

:::

**The Problem:**

Players must be able to find and connect to each other globally without relying on a static, centralized discovery server.

**Motivation:**

**Deliverables:**

- Integrate Iroh's Distributed Hash Table (DHT) for peer-to-peer node discovery.
- Handle automatic NAT traversal and hole-punching for global P2P connectivity.

## Project Deployment

### Desktop Applicatoin

::: danger

TODO: not finalized

:::

**The Problem:**

**Motivation:**

**Deliverables:**

### Mobile Application

::: danger

TODO: not finalized

:::

**The Problem:**

**Motivation:**

**Deliverables:**

### Deployment Ready Backend

::: danger

TODO: not finalized

:::

**The Problem:**

Difficult to connect from a third device to the backend.

**Motivation:**

**Deliverables:**

### Backend inside frontend

::: danger

TODO: not finalized

:::

**The Problem:**

**Motivation:**

**Deliverables:**

### Serving a demo website

::: danger

TODO: not finalized

:::

**The Problem:**

**Motivation:**

**Deliverables:**
