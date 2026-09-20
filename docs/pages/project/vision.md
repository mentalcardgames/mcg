---
outline: deep
---

# Vision & Baseline

To bridge our day-to-day software engineering with long-term objectives, the technical work is organized around core milestones. This page documents our **long-term goal & vision** alongside the **current software baseline** for each domain, providing a clear starting point for new contributions.

## 1. Domain-Specific Language & Game Engine (CGDL)

To transition from a single hardcoded card game to a general framework, we require a system that compiles and executes arbitrary game logic.

* **Long-Term Goal & Vision:** A completely generalized, dynamic game engine. Creators will define any card game (including custom rules, deck layouts, turn ordering, and victory conditions) in our bespoke **Card Game Description Language (CGDL)**. The engine compiles this code into a deterministic Finite State Machine (FSM) and executes it seamlessly.
* **Current Software Baseline:** We have a functional parser and parser-generator for CGDL (located under `/crates/cardgamedsl`). However, the main runtime application currently relies on a statically hardcoded state machine built strictly for a single Poker game. Decoupling this static logic is a primary technical goal.

## 2. Decoupled & Dynamic Frontend Interfaces

As the underlying game engine transitions to supporting dynamic rules, the user interface must adapt automatically to compile-time definitions without manual layout updates.

* **Long-Term Goal & Vision:** A fully dynamic, plug-and-play UI client. Rather than hardcoding visual elements, the frontend client will query the active game state from the FSM engine and dynamically render the appropriate game zones (e.g., table layouts, players' hands, face-down draw decks) and interactive action choices available to the current player.
* **Current Software Baseline:** Our current client UI (built in `egui`) has its layout and visual components hardcoded strictly for a Poker game. We have a robust screen routing system (`ScreenDef` / `ScreenWidget` registry) that serves as the foundation for this upcoming visual decoupling.

## 3. Zero-Knowledge Card Cryptography

Mental card games rely on mathematical proofs to ensure fair play, eliminating the need to trust a central host or other players.

* **Long-Term Goal & Vision:** A fully serverless, mathematically fair card system. Cards are shuffled, drawn, and held face-down using cryptographic primitives (such as commutative encryption and Zero-Knowledge Proofs). Players can prove they shuffled honestly or drew a card within game bounds without revealing their secret hands, ensuring total fairness even on hostile networks.
* **Current Software Baseline:** The current lobby system operates under an authoritative host model. The backend server holds full knowledge of the card deck and game state, acting as a trusted coordinator. We are transitioning toward implementing verifiable shuffling protocols using libraries like `arkworks`.

### Cryptography Abstraction

Since multiple ZKP protocols will be implemented to support the various mental card game requirements, the cryptography layer is heavily abstracted behind **Traits**. The Game Engine interacts exclusively with these traits rather than concrete implementations, allowing for flexibility, swapping of protocols, and cleaner testing.

## 4. Peer-to-Peer Consensual Networking

Distributed gameplay requires robust connection bootstrapping, peer discovery, and consensus on state transitions.

* **Long-Term Goal & Vision:** A serverless, resilient peer-to-peer (P2P) network mesh. Every player runs a local "Split Node" containing a frontend client and backend engine. Nodes connect directly using robust NAT traversal (hole punching) and run distributed consensus algorithms to agree on turns, message orderings, and state validations without relying on central infrastructure.
* **Current Software Baseline:** Sockets and QR-code lobbies communicating via a simplified coordinate-broker model. We are actively integrating **Iroh** for decentralized peer discovery via Distributed Hash Tables (DHT) and reliable socket connections.
